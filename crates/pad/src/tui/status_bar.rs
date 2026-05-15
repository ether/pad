use crate::buffer::Buffer;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;

#[derive(Debug, Clone, Copy)]
pub struct ShareBadge {
    pub author_count: usize,
}

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    buffer: &Buffer,
    file_label: &str,
    share: Option<ShareBadge>,
) {
    let dirty = if buffer.is_dirty() { "[modified] " } else { "" };
    let pos = buffer.cursor();
    let share_part = match share {
        Some(b) => format!("  Shared • you +{}", b.author_count.saturating_sub(1)),
        None => String::new(),
    };
    let line = format!(
        "  {dirty}{file_label}    line {}, col {}{}",
        pos.line + 1,
        pos.col + 1,
        share_part,
    );
    let p = Paragraph::new(line).style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(p, area);
}
