use crate::buffer::{Buffer, CursorPos};
use crate::tui::softwrap;
use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame<'_>, area: Rect, buffer: &Buffer) {
    let width = area.width;
    let mut visual_lines: Vec<Line<'_>> = Vec::new();
    let mut cursor_visual: Option<(u16, u16)> = None;
    let CursorPos {
        line: cline,
        col: ccol,
    } = buffer.cursor();

    for li in 0..buffer.line_count() {
        let line = buffer.line(li);
        let wrapped = softwrap::wrap_line(&line, width);
        for (offset_in_logical, slice) in wrapped {
            if li == cline
                && ccol >= offset_in_logical
                && ccol <= offset_in_logical + slice.chars().count()
            {
                let col = (ccol - offset_in_logical) as u16;
                let row = visual_lines.len() as u16;
                cursor_visual = Some((col, row));
            }
            visual_lines.push(Line::from(slice));
        }
    }
    let para = Paragraph::new(visual_lines).style(Style::default().fg(Color::Reset));
    frame.render_widget(para, area);

    if let Some((col, row)) = cursor_visual {
        let cx = area.x.saturating_add(col);
        let cy = area.y.saturating_add(row);
        if cx < area.right() && cy < area.bottom() {
            frame.set_cursor_position(Position::new(cx, cy));
        }
    }
}
