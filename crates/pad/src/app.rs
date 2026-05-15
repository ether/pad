use crate::buffer::Buffer;
use crate::buffer::sidecar::{PendingEntry, PendingLog, SidecarHandle};
use crate::cli::Mode;
use crate::config::paths;
use crate::keymap::KeyAction;
use crate::share::ShareState;
use crate::tui::Tui;
use std::path::PathBuf;
use std::time::Duration;

pub enum AppState {
    Editing,
    SaveAsPrompt(String),
    DirtyExitPrompt,
    SearchPrompt(String),
    ReplaceFromPrompt(String),
    ReplaceToPrompt {
        from: String,
        to: String,
    },
    GotoLinePrompt(String),
    InsertFilePrompt(String),
    HelpOverlay,
    FlashMessage(String),
    CollisionPrompt {
        remote_text: String,
        remote_base: String,
        pad_id: String,
    },
    SharePadNamePrompt(String),
    ShareOverlay {
        url: String,
        qr: String,
    },
}

/// Half-built share state — connect succeeded, but the remote had content and
/// we're waiting for the user's E/D/C choice.
pub struct PendingShare {
    pub remote_base: String,
    pub pad_id: String,
    pub handles: crate::share::network::NetworkHandles,
}

pub struct App {
    pub buffer: Buffer,
    pub sidecar: SidecarHandle,
    pub pending_log: PendingLog,
    pub file_path: Option<PathBuf>,
    pub file_label: String,
    pub state: AppState,
    pub share: Option<ShareState>,
    pub pending_share: Option<PendingShare>,
    pub show_authors: bool,
    quit_requested: bool,
}

impl App {
    pub fn from_mode(mode: Mode) -> anyhow::Result<Self> {
        let state_root = paths::state_root();
        std::fs::create_dir_all(&state_root)?;
        let (buffer, sidecar, file_path, file_label) = match mode {
            Mode::Untitled => {
                let sc = SidecarHandle::new_untitled(&state_root)?;
                (Buffer::empty(), sc, None, "New Buffer".to_string())
            }
            Mode::OpenFile(path) => {
                let sc = SidecarHandle::for_file(&state_root, &path)?;
                let buf = Buffer::load_from_file(&path)?;
                let label = path.display().to_string();
                (buf, sc, Some(path), label)
            }
            Mode::Recover | Mode::Restore | Mode::Setup | Mode::JoinUrl(_) => {
                unreachable!("dispatched by main()")
            }
        };
        let pending_log = PendingLog::open(&sidecar)?;
        Ok(Self {
            buffer,
            sidecar,
            pending_log,
            file_path,
            file_label,
            state: AppState::Editing,
            share: None,
            pending_share: None,
            show_authors: false,
            quit_requested: false,
        })
    }

    pub async fn run(&mut self, tui: &mut Tui) -> anyhow::Result<()> {
        let mut keys = crate::input::spawn_event_task();
        while !self.quit_requested {
            // Drain inbound + presence from share (non-blocking) before drawing.
            self.drain_share_channels()?;

            let prompt: Option<(&str, &str)> = match &self.state {
                AppState::SaveAsPrompt(input) => Some(("File Name to Write", input.as_str())),
                AppState::DirtyExitPrompt => Some(("Save modified buffer? (Y/N, ^C cancel)", "")),
                AppState::SearchPrompt(input) => Some(("Search", input.as_str())),
                AppState::ReplaceFromPrompt(input) => Some(("Search (to replace)", input.as_str())),
                AppState::ReplaceToPrompt { to, .. } => Some(("Replacement", to.as_str())),
                AppState::GotoLinePrompt(input) => Some(("Goto line", input.as_str())),
                AppState::InsertFilePrompt(input) => Some(("File to insert", input.as_str())),
                AppState::FlashMessage(msg) => Some(("", msg.as_str())),
                AppState::CollisionPrompt { .. } => Some((
                    "Pad already has content",
                    "[E]dit existing  [D]ifferent name  [C]ancel",
                )),
                AppState::SharePadNamePrompt(input) => Some(("Pad name", input.as_str())),
                _ => None,
            };
            let show_help = matches!(self.state, AppState::HelpOverlay);
            let share_overlay = if let AppState::ShareOverlay { url, qr } = &self.state {
                Some((url.as_str(), qr.as_str()))
            } else {
                None
            };
            let share_badge = self
                .share
                .as_ref()
                .map(|s| crate::tui::status_bar::ShareBadge {
                    author_count: s.authors.len(),
                });
            let authors_vec: Vec<String>;
            let authors_arg = if self.show_authors {
                if let Some(s) = &self.share {
                    authors_vec = s.authors.iter().cloned().collect();
                    Some((authors_vec.as_slice(), s.author_id.as_str()))
                } else {
                    None
                }
            } else {
                None
            };
            tui.draw(crate::tui::DrawInputs {
                buffer: &self.buffer,
                file_label: &self.file_label,
                prompt,
                show_help,
                share: share_badge,
                share_overlay,
                authors: authors_arg,
            })?;

            let tick = tokio::time::sleep(Duration::from_millis(50));
            tokio::pin!(tick);
            tokio::select! {
                Some(action) = keys.recv() => { self.handle(action).await?; }
                _ = &mut tick => {}
            }
        }
        Ok(())
    }

    fn drain_share_channels(&mut self) -> anyhow::Result<()> {
        // Take share out, drain, put back — avoids overlapping borrows of self.
        let Some(mut share) = self.share.take() else {
            return Ok(());
        };
        while let Ok(cs) = share.inbound_rx.try_recv() {
            if share.outbound.pending_len() > 50 {
                let _ = self.sidecar.pre_merge_snapshot(&self.buffer);
            }
            crate::share::inbound::apply_remote(&mut self.buffer, &cs, &share.outbound)?;
        }
        while let Ok(p) = share.presence_rx.try_recv() {
            match p {
                crate::share::network::PresenceEvent::Join { author_id, .. } => {
                    share.authors.insert(author_id);
                }
                crate::share::network::PresenceEvent::Leave { author_id } => {
                    share.authors.remove(&author_id);
                }
            }
        }
        self.share = Some(share);
        Ok(())
    }

    async fn handle(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match &self.state {
            AppState::Editing => self.handle_editing(action).await,
            AppState::SaveAsPrompt(_) => self.handle_save_prompt(action),
            AppState::DirtyExitPrompt => self.handle_dirty_prompt(action),
            AppState::SearchPrompt(_) => self.handle_search_prompt(action),
            AppState::ReplaceFromPrompt(_) => self.handle_replace_from(action),
            AppState::ReplaceToPrompt { .. } => self.handle_replace_to(action),
            AppState::GotoLinePrompt(_) => self.handle_goto_line(action),
            AppState::InsertFilePrompt(_) => self.handle_insert_file(action),
            AppState::CollisionPrompt { .. } => self.handle_collision(action).await,
            AppState::SharePadNamePrompt(_) => self.handle_share_pad_name(action).await,
            AppState::ShareOverlay { .. } => {
                self.state = AppState::Editing;
                Box::pin(self.handle_editing(action)).await
            }
            AppState::HelpOverlay | AppState::FlashMessage(_) => {
                self.state = AppState::Editing;
                Box::pin(self.handle_editing(action)).await
            }
        }
    }

    async fn handle_editing(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match action {
            KeyAction::InsertChar(c) => {
                self.buffer.snapshot_for_undo();
                let pre_len = self.buffer.text_len();
                let pre_offset = self.buffer.cursor_offset();
                self.pending_log.append(&PendingEntry::Insert {
                    offset: pre_offset,
                    text: c.to_string(),
                })?;
                self.buffer.insert_char(c);
                if let Some(share) = self.share.as_mut() {
                    let cs = crate::share::bridge::changeset_for_insert(
                        pre_len,
                        pre_offset,
                        &c.to_string(),
                    );
                    share.outbound.send(cs)?;
                }
            }
            KeyAction::Backspace => {
                let off = self.buffer.cursor_offset();
                if off > 0 {
                    self.buffer.snapshot_for_undo();
                    let pre_len = self.buffer.text_len();
                    let deleted = self
                        .buffer
                        .text()
                        .chars()
                        .nth((off - 1) as usize)
                        .map(|c| c.to_string())
                        .unwrap_or_default();
                    self.pending_log.append(&PendingEntry::Delete {
                        offset: off - 1,
                        len: 1,
                    })?;
                    self.buffer.backspace();
                    if let Some(share) = self.share.as_mut() {
                        let cs =
                            crate::share::bridge::changeset_for_delete(pre_len, off - 1, deleted);
                        share.outbound.send(cs)?;
                    }
                }
            }
            KeyAction::DeleteForward => {
                self.buffer.snapshot_for_undo();
                let pre_len = self.buffer.text_len();
                let off = self.buffer.cursor_offset();
                if off < pre_len {
                    let deleted = self
                        .buffer
                        .text()
                        .chars()
                        .nth(off as usize)
                        .map(|c| c.to_string())
                        .unwrap_or_default();
                    self.pending_log.append(&PendingEntry::Delete {
                        offset: off,
                        len: 1,
                    })?;
                    self.buffer.delete_char_forward();
                    if let Some(share) = self.share.as_mut() {
                        let cs = crate::share::bridge::changeset_for_delete(pre_len, off, deleted);
                        share.outbound.send(cs)?;
                    }
                }
            }
            KeyAction::Left => self.buffer.move_left(),
            KeyAction::Right => self.buffer.move_right(),
            KeyAction::Up => self.buffer.move_up(),
            KeyAction::Down => self.buffer.move_down(),
            KeyAction::WriteOut => {
                if let Some(p) = self.file_path.clone() {
                    self.buffer.save_to_file(&p)?;
                    self.pending_log.truncate()?;
                } else {
                    self.state = AppState::SaveAsPrompt(String::new());
                }
            }
            KeyAction::Exit => {
                if self.buffer.is_dirty() {
                    self.state = AppState::DirtyExitPrompt;
                } else {
                    self.quit_requested = true;
                }
            }
            KeyAction::Cut => {
                self.buffer.snapshot_for_undo();
                // Local-only change for now — cut spans a whole line; share
                // wire-up for multi-char deletes lands in a polish pass.
                self.buffer.cut_line();
            }
            KeyAction::Uncut => {
                self.buffer.snapshot_for_undo();
                self.buffer.uncut();
            }
            KeyAction::WhereIs => self.state = AppState::SearchPrompt(String::new()),
            KeyAction::Replace => self.state = AppState::ReplaceFromPrompt(String::new()),
            KeyAction::GotoLine => self.state = AppState::GotoLinePrompt(String::new()),
            KeyAction::InsertFile => self.state = AppState::InsertFilePrompt(String::new()),
            KeyAction::Undo => self.buffer.undo(),
            KeyAction::Redo => self.buffer.redo(),
            KeyAction::Help => self.state = AppState::HelpOverlay,
            KeyAction::CursorPos => {
                let pos = self.buffer.cursor();
                let total = self.buffer.line_count();
                self.state = AppState::FlashMessage(format!(
                    "line {}/{}, col {}",
                    pos.line + 1,
                    total,
                    pos.col + 1
                ));
            }
            KeyAction::Suspend => {
                #[cfg(unix)]
                {
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    unsafe {
                        libc::raise(libc::SIGTSTP);
                    }
                }
            }
            KeyAction::Share => {
                if let Some(share) = self.share.take() {
                    share.unshare();
                    self.state = AppState::FlashMessage("Unshared.".into());
                } else {
                    let remote = self.persisted_remote();
                    match remote {
                        Some(r) => self.start_share(&r).await?,
                        None => {
                            self.state = AppState::FlashMessage(
                                "No remote configured. Run 'pad --setup' first.".into(),
                            );
                        }
                    }
                }
            }
            KeyAction::ToggleAuthors => self.show_authors = !self.show_authors,
            KeyAction::CopyShareUrl => {
                if let Some(share) = &self.share {
                    let url = share.share_url();
                    let res = crate::share::osc52::copy_to_clipboard(&url);
                    self.state = AppState::FlashMessage(if res.is_ok() {
                        "URL copied (OSC 52)".into()
                    } else {
                        format!("URL: {url}")
                    });
                }
            }
            KeyAction::ReshowQr => {
                if let Some(share) = &self.share {
                    let url = share.share_url();
                    let qr = crate::share::qr::ansi(&url);
                    self.state = AppState::ShareOverlay { url, qr };
                }
            }
            KeyAction::Unbound => {}
        }
        Ok(())
    }

    fn persisted_remote(&self) -> Option<String> {
        crate::config::Config::load().ok().and_then(|c| c.remote)
    }

    fn derive_pad_id(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(short_random_pad_id)
    }

    pub async fn start_share(&mut self, remote: &str) -> anyhow::Result<()> {
        let pad_id = self.derive_pad_id();
        self.start_share_with_pad_id(remote, &pad_id).await
    }

    /// Build an App that joins a remote pad. Buffer is seeded from remote's
    /// initial text. No collision check — joining means the remote IS the truth.
    pub async fn from_join_url(p: crate::share::url_parse::ParsedPadUrl) -> anyhow::Result<Self> {
        let mut app = Self::from_mode(Mode::Untitled)?;
        let handles = crate::share::network::connect(&p.remote_base, &p.pad_id)
            .await
            .map_err(|e| anyhow::anyhow!("connect: {}", e.message))?;
        app.attach_share(
            &p.remote_base,
            &p.pad_id,
            handles,
            /*seed=*/ false,
            /*from_remote=*/ true,
        )
        .await?;
        Ok(app)
    }

    pub async fn start_share_with_pad_id(
        &mut self,
        remote: &str,
        pad_id: &str,
    ) -> anyhow::Result<()> {
        let handles = crate::share::network::connect(remote, pad_id)
            .await
            .map_err(|e| anyhow::anyhow!("connect: {}", e.message))?;
        if crate::share::collision::would_collide(&handles.initial_text)
            && !self.buffer.text().is_empty()
        {
            self.pending_share = Some(PendingShare {
                remote_base: remote.to_string(),
                pad_id: pad_id.to_string(),
                handles,
            });
            let pending = self.pending_share.as_ref().unwrap();
            self.state = AppState::CollisionPrompt {
                remote_text: pending.handles.initial_text.clone(),
                remote_base: remote.to_string(),
                pad_id: pad_id.to_string(),
            };
            return Ok(());
        }
        self.attach_share(
            remote, pad_id, handles, /*seed=*/ true, /*from_remote=*/ false,
        )
        .await
    }

    pub async fn attach_share(
        &mut self,
        remote_base: &str,
        pad_id: &str,
        handles: crate::share::network::NetworkHandles,
        seed_with_local: bool,
        seed_buffer_from_remote: bool,
    ) -> anyhow::Result<()> {
        let _ = self.sidecar.pre_share_snapshot(&self.buffer);

        if seed_buffer_from_remote {
            self.buffer.replace_all_text(&handles.initial_text);
            self.buffer.mark_clean();
        }

        let mut outbound = crate::share::outbound::OutboundQueue::new(handles.outbound_tx);
        if seed_with_local && !self.buffer.text().is_empty() && handles.initial_text.is_empty() {
            let cs = crate::share::bridge::changeset_for_insert(0, 0, &self.buffer.text());
            outbound.send(cs)?;
        }

        let mut authors = std::collections::HashSet::new();
        authors.insert(handles.author_id.clone());

        let url = format!("{}/p/{}", remote_base.trim_end_matches('/'), pad_id);
        let qr = crate::share::qr::ansi(&url);

        self.share = Some(ShareState {
            pad_id: pad_id.into(),
            remote_base: remote_base.into(),
            author_id: handles.author_id,
            outbound,
            inbound_rx: handles.inbound_rx,
            presence_rx: handles.presence_rx,
            net_task: handles.task,
            authors,
        });
        self.state = AppState::ShareOverlay { url, qr };
        Ok(())
    }

    fn handle_save_prompt(&mut self, action: KeyAction) -> anyhow::Result<()> {
        let AppState::SaveAsPrompt(input) = &mut self.state else {
            return Ok(());
        };
        match action {
            KeyAction::InsertChar('\n') => {
                if input.is_empty() {
                    self.state = AppState::Editing;
                    return Ok(());
                }
                let path = PathBuf::from(input.clone());
                self.buffer.save_to_file(&path)?;
                self.pending_log.truncate()?;
                self.sidecar.set_file_path(path.clone())?;
                self.file_path = Some(path.clone());
                self.file_label = path.display().to_string();
                self.state = AppState::Editing;
            }
            KeyAction::InsertChar(c) => input.push(c),
            KeyAction::Backspace => {
                input.pop();
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }

    fn handle_dirty_prompt(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match action {
            KeyAction::InsertChar('y') | KeyAction::InsertChar('Y') => {
                if let Some(p) = self.file_path.clone() {
                    self.buffer.save_to_file(&p)?;
                    self.pending_log.truncate()?;
                    self.quit_requested = true;
                } else {
                    self.state = AppState::SaveAsPrompt(String::new());
                }
            }
            KeyAction::InsertChar('n') | KeyAction::InsertChar('N') => {
                self.quit_requested = true;
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }

    fn handle_search_prompt(&mut self, action: KeyAction) -> anyhow::Result<()> {
        let AppState::SearchPrompt(input) = &mut self.state else {
            return Ok(());
        };
        match action {
            KeyAction::InsertChar('\n') => {
                let needle = input.clone();
                self.state = AppState::Editing;
                if let Some(pos) = self.buffer.search_forward(&needle) {
                    self.buffer.move_cursor_to(pos);
                }
            }
            KeyAction::InsertChar(c) => input.push(c),
            KeyAction::Backspace => {
                input.pop();
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }

    fn handle_replace_from(&mut self, action: KeyAction) -> anyhow::Result<()> {
        let AppState::ReplaceFromPrompt(input) = &mut self.state else {
            return Ok(());
        };
        match action {
            KeyAction::InsertChar('\n') => {
                let from = input.clone();
                self.state = AppState::ReplaceToPrompt {
                    from,
                    to: String::new(),
                };
            }
            KeyAction::InsertChar(c) => input.push(c),
            KeyAction::Backspace => {
                input.pop();
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }

    fn handle_replace_to(&mut self, action: KeyAction) -> anyhow::Result<()> {
        let AppState::ReplaceToPrompt { from, to } = &mut self.state else {
            return Ok(());
        };
        match action {
            KeyAction::InsertChar('\n') => {
                let (from, to) = (from.clone(), to.clone());
                self.buffer.snapshot_for_undo();
                let _n = self.buffer.replace_all(&from, &to);
                self.state = AppState::Editing;
            }
            KeyAction::InsertChar(c) => to.push(c),
            KeyAction::Backspace => {
                to.pop();
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }

    fn handle_goto_line(&mut self, action: KeyAction) -> anyhow::Result<()> {
        let AppState::GotoLinePrompt(input) = &mut self.state else {
            return Ok(());
        };
        match action {
            KeyAction::InsertChar('\n') => {
                if let Ok(n) = input.trim().parse::<usize>() {
                    let target = n.saturating_sub(1);
                    self.buffer.move_cursor_to(crate::buffer::CursorPos {
                        line: target,
                        col: 0,
                    });
                }
                self.state = AppState::Editing;
            }
            KeyAction::InsertChar(c) if c.is_ascii_digit() => input.push(c),
            KeyAction::Backspace => {
                input.pop();
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_file(&mut self, action: KeyAction) -> anyhow::Result<()> {
        let AppState::InsertFilePrompt(input) = &mut self.state else {
            return Ok(());
        };
        match action {
            KeyAction::InsertChar('\n') => {
                let path = std::path::PathBuf::from(input.clone());
                self.state = AppState::Editing;
                if let Ok(bytes) = std::fs::read(&path)
                    && let Ok(text) = std::str::from_utf8(&bytes)
                {
                    self.buffer.snapshot_for_undo();
                    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                    self.buffer.insert_str(&normalized);
                }
            }
            KeyAction::InsertChar(c) => input.push(c),
            KeyAction::Backspace => {
                input.pop();
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }

    async fn handle_collision(&mut self, action: KeyAction) -> anyhow::Result<()> {
        let prev = std::mem::replace(&mut self.state, AppState::Editing);
        let AppState::CollisionPrompt {
            remote_text,
            remote_base,
            pad_id,
        } = prev
        else {
            return Ok(());
        };
        match action {
            KeyAction::InsertChar('e') | KeyAction::InsertChar('E') => {
                let _ = self.sidecar.pre_share_snapshot(&self.buffer);
                self.buffer.replace_all_text(&remote_text);
                let Some(pending) = self.pending_share.take() else {
                    return Ok(());
                };
                self.attach_share(
                    &pending.remote_base,
                    &pending.pad_id,
                    pending.handles,
                    /*seed=*/ false,
                    /*from_remote=*/ false,
                )
                .await?;
            }
            KeyAction::InsertChar('d') | KeyAction::InsertChar('D') => {
                if let Some(p) = self.pending_share.take() {
                    p.handles.task.abort();
                }
                self.state = AppState::SharePadNamePrompt(String::new());
            }
            _ => {
                if let Some(p) = self.pending_share.take() {
                    p.handles.task.abort();
                }
                let _ = (remote_base, pad_id);
            }
        }
        Ok(())
    }

    async fn handle_share_pad_name(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match action {
            KeyAction::InsertChar('\n') => {
                let new_pad_id = match &self.state {
                    AppState::SharePadNamePrompt(s) => s.clone(),
                    _ => return Ok(()),
                };
                self.state = AppState::Editing;
                if !new_pad_id.is_empty()
                    && let Some(remote) = self.persisted_remote()
                {
                    Box::pin(self.start_share_with_pad_id(&remote, &new_pad_id)).await?;
                }
            }
            KeyAction::InsertChar(c) => {
                if let AppState::SharePadNamePrompt(input) = &mut self.state {
                    input.push(c);
                }
            }
            KeyAction::Backspace => {
                if let AppState::SharePadNamePrompt(input) = &mut self.state {
                    input.pop();
                }
            }
            KeyAction::Exit => self.state = AppState::Editing,
            _ => {}
        }
        Ok(())
    }
}

fn short_random_pad_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789"
        .chars()
        .collect();
    let mut out = String::with_capacity(12);
    let mut t = ts;
    for _ in 0..12 {
        out.push(chars[(t as usize) % chars.len()]);
        t /= chars.len() as u128;
    }
    out
}
