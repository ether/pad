use crate::buffer::Buffer;
use crate::buffer::sidecar::{PendingEntry, PendingLog, SidecarHandle};
use crate::cli::Mode;
use crate::config::paths;
use crate::input;
use crate::keymap::KeyAction;
use crate::tui::Tui;
use std::path::PathBuf;
use std::time::Duration;

pub enum AppState {
    Editing,
    SaveAsPrompt(String),
    DirtyExitPrompt,
    SearchPrompt(String),
    ReplaceFromPrompt(String),
    ReplaceToPrompt { from: String, to: String },
    GotoLinePrompt(String),
    InsertFilePrompt(String),
    HelpOverlay,
    FlashMessage(String),
}

pub struct App {
    pub buffer: Buffer,
    pub sidecar: SidecarHandle,
    pub pending_log: PendingLog,
    pub file_path: Option<PathBuf>,
    pub file_label: String,
    pub state: AppState,
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
            Mode::Recover => unreachable!("Recover mode dispatches through recover::run"),
        };
        let pending_log = PendingLog::open(&sidecar)?;
        Ok(Self {
            buffer,
            sidecar,
            pending_log,
            file_path,
            file_label,
            state: AppState::Editing,
            quit_requested: false,
        })
    }

    pub fn run(&mut self, tui: &mut Tui) -> anyhow::Result<()> {
        while !self.quit_requested {
            let prompt: Option<(&str, &str)> = match &self.state {
                AppState::SaveAsPrompt(input) => Some(("File Name to Write", input.as_str())),
                AppState::DirtyExitPrompt => Some(("Save modified buffer? (Y/N, ^C cancel)", "")),
                AppState::SearchPrompt(input) => Some(("Search", input.as_str())),
                AppState::ReplaceFromPrompt(input) => Some(("Search (to replace)", input.as_str())),
                AppState::ReplaceToPrompt { to, .. } => Some(("Replacement", to.as_str())),
                AppState::GotoLinePrompt(input) => Some(("Goto line", input.as_str())),
                AppState::InsertFilePrompt(input) => Some(("File to insert", input.as_str())),
                AppState::FlashMessage(msg) => Some(("", msg.as_str())),
                _ => None,
            };
            let show_help = matches!(self.state, AppState::HelpOverlay);
            tui.draw_app(&self.buffer, &self.file_label, prompt, show_help)?;
            if let Some(action) = input::next_action(Duration::from_millis(50))? {
                self.handle(action)?;
            }
        }
        Ok(())
    }

    fn handle(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match &self.state {
            AppState::Editing => self.handle_editing(action),
            AppState::SaveAsPrompt(_) => self.handle_save_prompt(action),
            AppState::DirtyExitPrompt => self.handle_dirty_prompt(action),
            AppState::SearchPrompt(_) => self.handle_search_prompt(action),
            AppState::ReplaceFromPrompt(_) => self.handle_replace_from(action),
            AppState::ReplaceToPrompt { .. } => self.handle_replace_to(action),
            AppState::GotoLinePrompt(_) => self.handle_goto_line(action),
            AppState::InsertFilePrompt(_) => self.handle_insert_file(action),
            AppState::HelpOverlay | AppState::FlashMessage(_) => {
                // Dismiss the overlay AND apply the dismissing keystroke — otherwise
                // sequences like `^C` (cursor pos) → `^X` (exit) would silently swallow
                // the ^X. Plain InsertChars get re-applied too, matching nano's "any
                // key dismisses" plus the user's likely intent.
                self.state = AppState::Editing;
                self.handle_editing(action)
            }
        }
    }

    fn handle_editing(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match action {
            KeyAction::InsertChar(c) => {
                self.buffer.snapshot_for_undo();
                self.pending_log.append(&PendingEntry::Insert {
                    offset: self.buffer.cursor_offset(),
                    text: c.to_string(),
                })?;
                self.buffer.insert_char(c);
            }
            KeyAction::Backspace => {
                let off = self.buffer.cursor_offset();
                if off > 0 {
                    self.buffer.snapshot_for_undo();
                    self.pending_log.append(&PendingEntry::Delete {
                        offset: off - 1,
                        len: 1,
                    })?;
                }
                self.buffer.backspace();
            }
            KeyAction::DeleteForward => {
                self.buffer.snapshot_for_undo();
                let off = self.buffer.cursor_offset();
                self.pending_log.append(&PendingEntry::Delete {
                    offset: off,
                    len: 1,
                })?;
                self.buffer.delete_char_forward();
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
            KeyAction::Unbound => {}
        }
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
}
