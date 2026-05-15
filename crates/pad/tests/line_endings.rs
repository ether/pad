use pad::buffer::{Buffer, LineEnding};

#[test]
fn detect_lf() {
    assert_eq!(LineEnding::detect("a\nb\nc"), LineEnding::Lf);
}

#[test]
fn detect_crlf() {
    assert_eq!(LineEnding::detect("a\r\nb\r\n"), LineEnding::Crlf);
}

#[test]
fn detect_cr_only() {
    assert_eq!(LineEnding::detect("a\rb\rc"), LineEnding::Cr);
}

#[test]
fn default_to_lf_when_no_newlines() {
    assert_eq!(LineEnding::detect("hello"), LineEnding::Lf);
}

#[test]
fn mixed_picks_most_common() {
    assert_eq!(LineEnding::detect("a\r\nb\r\nc\nd"), LineEnding::Crlf);
}

#[test]
fn buffer_roundtrips_crlf() {
    let original = "a\r\nb\r\nc\r\n";
    let mut b = Buffer::from_text_with_ending(original);
    assert_eq!(b.line_ending(), LineEnding::Crlf);
    b.insert_char('!');
    assert_eq!(b.line_ending(), LineEnding::Crlf);
    assert!(b.serialize_for_save().starts_with("!a\r\nb\r\n"));
}
