use pad::buffer::Buffer;

#[test]
fn text_len_tracks_mutations() {
    let mut b = Buffer::empty();
    assert_eq!(b.text_len(), 0);
    b.insert_char('a');
    assert_eq!(b.text_len(), 1);
    b.insert_str("bc");
    assert_eq!(b.text_len(), 3);
    b.backspace();
    assert_eq!(b.text_len(), 2);
}

#[test]
fn replace_all_text_resets_rope_and_dirties() {
    let mut b = Buffer::from_text("original");
    assert!(!b.is_dirty());
    b.replace_all_text("replaced");
    assert_eq!(b.text(), "replaced");
    assert!(b.is_dirty());
}
