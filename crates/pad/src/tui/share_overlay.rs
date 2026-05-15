use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect, url: &str, qr_ansi: &str) {
    let body = format!(
        "Shared at:\n  {url}\n\n{qr_ansi}\n\n[M-S to unshare]   [M-C copy URL]   any key dismiss",
    );
    let p = Paragraph::new(body)
        .block(Block::default().title(" Share ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Reset));
    frame.render_widget(p, area);
}
