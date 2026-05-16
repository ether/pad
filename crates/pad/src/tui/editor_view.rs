use crate::buffer::{Buffer, CursorPos};
use crate::tui::softwrap;
use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

/// Render the editor view. `scroll` is the index of the FIRST visual row to
/// display (top-of-viewport). The function auto-adjusts `scroll` so the
/// cursor stays inside the visible region — without this, filling the pad
/// past the terminal height would leave the caret marooned off-screen with
/// no way to see what was being typed.
pub fn render(frame: &mut Frame<'_>, area: Rect, buffer: &Buffer, scroll: &mut u16) {
    let width = area.width;
    let height = area.height;
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

    // Auto-scroll: if the cursor's visual row sits outside the current
    // viewport, slide the viewport to include it. Clamp `scroll` so we
    // never leave blank space at the bottom while content's still above.
    if let Some((_, cursor_row)) = cursor_visual {
        if cursor_row < *scroll {
            *scroll = cursor_row;
        } else if height > 0 && cursor_row >= scroll.saturating_add(height) {
            *scroll = cursor_row - height + 1;
        }
    }
    let total_visual = visual_lines.len() as u16;
    let max_scroll = total_visual.saturating_sub(height);
    if *scroll > max_scroll {
        *scroll = max_scroll;
    }

    // Render only the visible slice. ratatui's Paragraph supports a `.scroll`
    // method that hides leading rows; using it lets the paragraph layout
    // (incl. cursor positioning) match what the user sees.
    let para = Paragraph::new(visual_lines)
        .style(Style::default().fg(Color::Reset))
        .scroll((*scroll, 0));
    frame.render_widget(para, area);

    if let Some((col, row)) = cursor_visual {
        let visible_row = row.saturating_sub(*scroll);
        let cx = area.x.saturating_add(col);
        let cy = area.y.saturating_add(visible_row);
        if cx < area.right() && cy < area.bottom() && row >= *scroll {
            frame.set_cursor_position(Position::new(cx, cy));
        }
    }
}
