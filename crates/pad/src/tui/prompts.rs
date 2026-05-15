use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render_prompt(frame: &mut Frame<'_>, area: Rect, label: &str, input: &str) {
    let text = if label.is_empty() {
        input.to_string()
    } else {
        format!("{label}: {input}_")
    };
    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Reset).bg(Color::DarkGray));
    frame.render_widget(p, area);
}
