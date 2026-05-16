use crate::tui::sanitize;
use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect, authors: &[String], self_id: &str) {
    let mut lines = Vec::new();
    for a in authors {
        // Author IDs are server-supplied — strip terminal escapes so a
        // hostile co-author can't inject ANSI via their userId. See
        // tui::sanitize.
        let safe = sanitize::for_terminal(a);
        let label = if a == self_id {
            format!("{safe} (you)")
        } else {
            safe.into_owned()
        };
        lines.push(label);
    }
    let text = lines.join("\n");
    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    let h = (authors.len() as u16 + 2).min(area.height);
    let w = 28u16.min(area.width);
    let region = Rect {
        x: inner.right().saturating_sub(w),
        y: inner.y,
        width: w,
        height: h,
    };
    let p = Paragraph::new(text)
        .block(Block::default().title(" Authors ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Reset).bg(Color::Black));
    frame.render_widget(p, region);
}
