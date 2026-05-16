pub mod author_overlay;
pub mod editor_view;
pub mod help;
pub mod prompts;
pub mod share_overlay;
pub mod softwrap;
pub mod status_bar;

use crate::buffer::Buffer;
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use std::io::Stdout;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

pub struct DrawInputs<'a> {
    pub buffer: &'a Buffer,
    pub file_label: &'a str,
    pub prompt: Option<(&'a str, &'a str)>,
    pub show_help: bool,
    pub share: Option<status_bar::ShareBadge>,
    pub share_overlay: Option<(&'a str, &'a str)>, // (url, qr)
    pub authors: Option<(&'a [String], &'a str)>,  // (authors, self_id)
}

impl Tui {
    pub fn enter() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn draw_app(
        &mut self,
        buffer: &Buffer,
        file_label: &str,
        prompt: Option<(&str, &str)>,
        show_help: bool,
    ) -> anyhow::Result<()> {
        self.draw(DrawInputs {
            buffer,
            file_label,
            prompt,
            show_help,
            share: None,
            share_overlay: None,
            authors: None,
        })
    }

    pub fn draw(&mut self, inputs: DrawInputs<'_>) -> anyhow::Result<()> {
        let DrawInputs {
            buffer,
            file_label,
            prompt,
            show_help,
            share,
            share_overlay,
            authors,
        } = inputs;
        self.terminal.draw(|frame| {
            let area = frame.area();
            let constraints: Vec<Constraint> = if prompt.is_some() {
                vec![
                    Constraint::Min(1),
                    Constraint::Length(2),
                    Constraint::Length(1),
                ]
            } else {
                vec![Constraint::Min(1), Constraint::Length(1)]
            };
            let chunks = Layout::vertical(constraints).split(area);
            editor_view::render(frame, chunks[0], buffer);
            if show_help {
                help::render(frame, chunks[0]);
            }
            if let Some((url, qr)) = share_overlay {
                share_overlay::render(frame, chunks[0], url, qr);
            }
            if let Some((authors, self_id)) = authors {
                author_overlay::render(frame, chunks[0], authors, self_id);
            }
            if let Some((label, input)) = prompt {
                prompts::render_prompt(frame, chunks[1], label, input);
                status_bar::render(frame, chunks[2], buffer, file_label, share);
            } else {
                status_bar::render(frame, chunks[1], buffer, file_label, share);
            }
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, DisableBracketedPaste, LeaveAlternateScreen);
    }
}
