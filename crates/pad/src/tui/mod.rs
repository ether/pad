pub mod editor_view;
pub mod help;
pub mod prompts;
pub mod softwrap;
pub mod status_bar;

use crate::buffer::Buffer;
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

impl Tui {
    pub fn enter() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
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
            if let Some((label, input)) = prompt {
                prompts::render_prompt(frame, chunks[1], label, input);
                status_bar::render(frame, chunks[2], buffer, file_label);
            } else {
                status_bar::render(frame, chunks[1], buffer, file_label);
            }
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}
