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
    // After 'a' + '\n' the rope is "a\n" with the cursor on the trailing
    // empty line. Typing 'b' there auto-appends a fresh trailing '\n' so the
    // doc keeps ending with '\n' — Etherpad's pad invariant. Without that
    // auto-append, the 'b' would be inserted past the existing '\n' and the
    // browser-side line assembler aborts ("line assembler not finished").
    b.insert_char('b');
    assert_eq!(b.text(), "a\nb\n");
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

#[test]
fn cut_then_uncut_round_trips_line() {
    let mut b = Buffer::from_text("alpha\nbravo\ncharlie");
    b.move_cursor_to(CursorPos { line: 1, col: 0 });
    b.cut_line();
    assert_eq!(b.text(), "alpha\ncharlie");
    b.uncut();
    assert_eq!(b.text(), "alpha\nbravo\ncharlie");
}

#[test]
fn search_finds_first_match_forward() {
    let mut b = Buffer::from_text("alpha\nbeta\nalpha");
    b.move_cursor_to(CursorPos { line: 0, col: 0 });
    let found = b.search_forward("alpha");
    assert_eq!(found, Some(CursorPos { line: 0, col: 0 }));
    b.move_cursor_to(CursorPos { line: 0, col: 1 });
    let next = b.search_forward("alpha");
    assert_eq!(next, Some(CursorPos { line: 2, col: 0 }));
}

#[test]
fn search_returns_none_when_missing() {
    let b = Buffer::from_text("alpha\nbeta");
    assert_eq!(b.search_forward("zeta"), None);
}

#[test]
fn move_to_line_start_and_end() {
    let mut b = Buffer::from_text("hello world\nsecond line\n");
    b.move_cursor_to(CursorPos { line: 0, col: 5 });
    b.move_to_line_start();
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 0 });
    b.move_to_line_end();
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 11 });
}

#[test]
fn move_to_line_end_clamps_at_actual_line_length() {
    // From a long line, move down to a shorter line — line_end must go to
    // the shorter line's actual end, not preserve the long line's col.
    let mut b = Buffer::from_text("longest line here\nshort\n");
    b.move_cursor_to(CursorPos { line: 1, col: 0 });
    b.move_to_line_end();
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 5 });
}

#[test]
fn move_to_document_start_and_end() {
    let mut b = Buffer::from_text("a\nb\nc\n");
    b.move_cursor_to(CursorPos { line: 2, col: 1 });
    b.move_to_document_start();
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 0 });
    b.move_to_document_end();
    // doc ends with '\n' → cursor sits at end of last content line,
    // not on the trailing-empty line.
    assert_eq!(b.cursor(), CursorPos { line: 2, col: 1 });
}

#[test]
fn insert_str_normalises_crlf_to_lf() {
    // Many terminals deliver bracketed-paste with '\r' line terminators
    // (or '\r\n' on Windows clipboards). The underlying rope must store
    // '\n' so Etherpad's pad ends up with real line breaks, not literal
    // '\r' characters that render as "cursor to col 0" but don't split
    // lines server-side. Normalize in insert_str so both the local
    // rope and the outbound changeset bank contain only '\n'.
    let mut b = Buffer::from_text("X\n");
    b.move_cursor_to(CursorPos { line: 0, col: 1 });
    let (_, actual) = b.insert_str("a\r\nb\rc");
    assert_eq!(actual, "a\nb\nc");
    assert_eq!(b.text(), "Xa\nb\nc\n");
}

#[test]
fn insert_str_at_trailing_empty_keeps_local_and_wire_in_sync() {
    // Regression for Wikipedia-paste bug: pasting a multi-char block at
    // the trailing-empty line via insert_str inserted the synth-'\n'
    // after the FIRST char in the wire accumulator but at the END of
    // the rope locally — server and local diverged, content visible
    // through the browser was right but the local buffer had a
    // different shape. The fix synthesizes the trailing '\n' once,
    // appended to the WHOLE pasted block, so the rope and the returned
    // accumulator are byte-identical.
    let mut b = Buffer::from_text("X\n");
    b.move_cursor_to(CursorPos { line: 1, col: 0 }); // trailing-empty
    let (pos, actual) = b.insert_str("Hello world");
    assert_eq!(pos, 2);
    assert_eq!(actual, "Hello world\n");
    assert_eq!(b.text(), "X\nHello world\n");
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 11 });
}

#[test]
fn insert_str_at_trailing_empty_with_internal_newlines() {
    let mut b = Buffer::from_text("X\n");
    b.move_cursor_to(CursorPos { line: 1, col: 0 });
    let (pos, actual) = b.insert_str("Hello\nworld");
    assert_eq!(pos, 2);
    assert_eq!(actual, "Hello\nworld\n");
    assert_eq!(b.text(), "X\nHello\nworld\n");
    assert_eq!(b.cursor(), CursorPos { line: 2, col: 5 });
}

#[test]
fn insert_str_at_trailing_empty_already_ends_with_newline() {
    let mut b = Buffer::from_text("X\n");
    b.move_cursor_to(CursorPos { line: 1, col: 0 });
    // s already ends with \n — don't double-synth.
    let (pos, actual) = b.insert_str("Hello world\n");
    assert_eq!(pos, 2);
    assert_eq!(actual, "Hello world\n");
    assert_eq!(b.text(), "X\nHello world\n");
}

#[test]
fn insert_str_in_middle_of_line() {
    let mut b = Buffer::from_text("abc\n");
    b.move_cursor_to(CursorPos { line: 0, col: 2 });
    let (pos, actual) = b.insert_str("XY");
    assert_eq!(pos, 2);
    assert_eq!(actual, "XY");
    assert_eq!(b.text(), "abXYc\n");
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 4 });
}

#[test]
fn insert_str_multiline_in_middle_of_line() {
    let mut b = Buffer::from_text("abc\n");
    b.move_cursor_to(CursorPos { line: 0, col: 2 });
    let (pos, actual) = b.insert_str("X\nY");
    assert_eq!(pos, 2);
    assert_eq!(actual, "X\nY");
    assert_eq!(b.text(), "abX\nYc\n");
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 1 });
}

#[test]
fn page_down_moves_n_lines() {
    let mut b = Buffer::from_text("a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n");
    b.move_to_document_start();
    b.page_down(3);
    assert_eq!(b.cursor(), CursorPos { line: 3, col: 0 });
    b.page_down(3);
    assert_eq!(b.cursor(), CursorPos { line: 6, col: 0 });
}

#[test]
fn page_down_clamps_to_last_meaningful_line() {
    // Doc ends with '\n' → cursor mustn't land on the trailing-empty
    // line, or subsequent typing would trip the trailing-\n synthesis
    // path. page_down clamps to line_count - 2 in that case.
    let mut b = Buffer::from_text("a\nb\nc\n");
    b.move_to_document_start();
    b.page_down(100);
    assert_eq!(b.cursor(), CursorPos { line: 2, col: 0 });
}

#[test]
fn page_up_clamps_to_zero() {
    let mut b = Buffer::from_text("a\nb\nc\nd\ne\n");
    b.move_cursor_to(CursorPos { line: 4, col: 0 });
    b.page_up(2);
    assert_eq!(b.cursor(), CursorPos { line: 2, col: 0 });
    b.page_up(100);
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 0 });
}

#[test]
fn move_to_document_end_no_trailing_newline() {
    let mut b = Buffer::from_text("a\nb");
    b.move_to_document_end();
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 1 });
}

#[test]
fn replace_one_replaces_first_match() {
    let mut b = Buffer::from_text("foo bar foo");
    b.replace_one("foo", "FOO");
    assert_eq!(b.text(), "FOO bar foo");
}

#[test]
fn replace_all_replaces_all_matches() {
    let mut b = Buffer::from_text("foo bar foo");
    let n = b.replace_all("foo", "FOO");
    assert_eq!(n, 2);
    assert_eq!(b.text(), "FOO bar FOO");
}

#[test]
fn undo_reverses_insert() {
    let mut b = Buffer::empty();
    b.snapshot_for_undo();
    b.insert_char('a');
    b.snapshot_for_undo();
    b.insert_char('b');
    b.undo();
    assert_eq!(b.text(), "a");
    b.undo();
    assert_eq!(b.text(), "");
}

#[test]
fn redo_replays_undone_edits() {
    let mut b = Buffer::empty();
    b.snapshot_for_undo();
    b.insert_char('a');
    b.snapshot_for_undo();
    b.insert_char('b');
    b.undo();
    b.redo();
    assert_eq!(b.text(), "ab");
}
