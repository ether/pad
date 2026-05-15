use pad::buffer::{Buffer, CursorPos};

#[test]
fn empty_buffer() {
    let b = Buffer::empty();
    assert_eq!(b.text(), "");
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 0 });
    assert!(!b.is_dirty());
}

#[test]
fn insert_char_advances_cursor_and_dirties() {
    let mut b = Buffer::empty();
    b.insert_char('a');
    assert_eq!(b.text(), "a");
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 1 });
    assert!(b.is_dirty());
}

#[test]
fn insert_newline_moves_to_next_line() {
    let mut b = Buffer::empty();
    b.insert_char('a');
    b.insert_char('\n');
    b.insert_char('b');
    assert_eq!(b.text(), "a\nb");
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 1 });
}

#[test]
fn backspace_at_start_of_line_joins_lines() {
    let mut b = Buffer::from_text("a\nb");
    b.move_cursor_to(CursorPos { line: 1, col: 0 });
    b.backspace();
    assert_eq!(b.text(), "ab");
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 1 });
}

#[test]
fn delete_char_at_end_of_line_joins() {
    let mut b = Buffer::from_text("ab\ncd");
    b.move_cursor_to(CursorPos { line: 0, col: 2 });
    b.delete_char_forward();
    assert_eq!(b.text(), "abcd");
}

#[test]
fn arrow_navigation_clamps_to_line() {
    let mut b = Buffer::from_text("abc\nxy");
    b.move_cursor_to(CursorPos { line: 0, col: 3 });
    b.move_down();
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 2 });
}

#[test]
fn from_text_preserves_initial_clean_state() {
    let b = Buffer::from_text("hello");
    assert_eq!(b.text(), "hello");
    assert!(!b.is_dirty());
}
