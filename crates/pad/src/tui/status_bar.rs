use crate::buffer::Buffer;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame<'_>, area: Rect, buffer: &Buffer, file_label: &str) {
    let dirty = if buffer.is_dirty() { "[modified] " } else { "" };
    let pos = buffer.cursor();
    let line = format!(
        "  {dirty}{file_label}    line {}, col {}",
        pos.line + 1,
        pos.col + 1
    );
    let p = Paragraph::new(line).style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(p, area);
}
