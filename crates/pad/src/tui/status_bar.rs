use crate::buffer::Buffer;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;

#[derive(Debug, Clone)]
pub struct ShareBadge {
    pub author_count: usize,
    /// Full pad URL — `<remote_base>/p/<pad_id>`. Always shown in the
    /// status bar when connected so the user can copy/paste it without
    /// hunting through menus or hitting a shortcut. The terminal also
    /// gets a click-to-copy via M-C, but having it on screen is the
    /// faster path.
    pub url: String,
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
    let share_part = match share.as_ref() {
        Some(b) => format!(
            "  Shared • you +{} • {}",
            b.author_count.saturating_sub(1),
            b.url,
        ),
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
