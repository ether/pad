use crate::buffer::{Buffer, CursorPos};
use crate::tui::{sanitize, softwrap};
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
        let raw_line = buffer.line(li);
        // Sanitize the WHOLE logical line first so softwrap and cursor
        // arithmetic agree on character positions. Wrapping the raw line
        // then sanitizing per visual slice would let injected control
        // chars shift the visual cursor relative to the displayed text
        // (a co-author injecting N controls before your column would
        // push your caret N cells right of the glyph). See tui::sanitize
        // for why we strip — TL;DR ratatui passes adjacent cells'
        // symbols through to the terminal unfiltered, so raw ESC bytes
        // in pad content are interpreted as escape sequences.
        let line_owned = sanitize::for_terminal(&raw_line).into_owned();
        let line: &str = &line_owned;
        // Cursor's column index needs to drop by however many control
        // chars existed strictly before `ccol` on the original line.
        let ccol_adjusted = if li == cline {
            let stripped_before = raw_line
                .chars()
                .take(ccol)
                .filter(|c| !(*c == '\n' || *c == '\t') && c.is_control())
                .count();
            ccol.saturating_sub(stripped_before)
        } else {
            ccol
        };
        let wrapped = softwrap::wrap_line(line, width);
        for (offset_in_logical, slice) in wrapped {
            if li == cline
                && ccol_adjusted >= offset_in_logical
                && ccol_adjusted <= offset_in_logical + slice.chars().count()
            {
                let col = (ccol_adjusted - offset_in_logical) as u16;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_buffer_to_terminal_grid(buf: &Buffer, width: u16, height: u16) -> Vec<String> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut scroll = 0u16;
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, width, height);
                render(frame, area, buf, &mut scroll);
            })
            .unwrap();
        let backend = terminal.backend();
        let buffer = backend.buffer();
        let mut rows = Vec::with_capacity(height as usize);
        for y in 0..height {
            let mut row = String::new();
            for x in 0..width {
                row.push_str(buffer.cell(Position::new(x, y)).unwrap().symbol());
            }
            rows.push(row);
        }
        rows
    }

    #[test]
    fn render_strips_esc_sequences_so_terminal_cant_interpret_them() {
        // If a hostile co-author wrote `\x1b]52;c;Zm9v\x07` into the pad,
        // ratatui's CrosstermBackend would concatenate the adjacent cells
        // and the host terminal would parse OSC 52 (silent clipboard
        // hijack). The render must strip those bytes so no rendered cell
        // contains a control char.
        let buf = Buffer::from_text("\x1b]52;c;Zm9v\x07hello\n");
        let rows = render_buffer_to_terminal_grid(&buf, 40, 4);
        for row in &rows {
            for c in row.chars() {
                assert!(
                    !c.is_control() || c == ' ',
                    "rendered grid still contains a control char: {c:?} in row {row:?}"
                );
            }
        }
        // The harmless ASCII tail of the OSC sequence + the actual text
        // SHOULD survive — they're not control chars themselves.
        assert!(rows[0].contains("]52;c;Zm9vhello"));
    }

    #[test]
    fn render_preserves_tabs_and_newlines() {
        let buf = Buffer::from_text("col1\tcol2\nrow2\n");
        let rows = render_buffer_to_terminal_grid(&buf, 40, 4);
        // Tab expansion: ratatui renders tab as a single cell containing
        // the tab character; the important thing is content lands on
        // separate rows.
        assert!(rows[0].contains("col1"));
        assert!(rows[0].contains("col2"));
        assert!(rows[1].contains("row2"));
    }
}
