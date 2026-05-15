use super::CursorPos;
use ropey::Rope;

pub fn search_forward(rope: &Rope, needle: &str, start_char_idx: usize) -> Option<CursorPos> {
    if needle.is_empty() {
        return None;
    }
    let text: String = rope.slice(start_char_idx..).into();
    let byte_pos = text.find(needle)?;
    let char_offset = text[..byte_pos].chars().count();
    let absolute = start_char_idx + char_offset;
    let line = rope.char_to_line(absolute);
    let line_start = rope.line_to_char(line);
    let col = absolute - line_start;
    Some(CursorPos { line, col })
}
