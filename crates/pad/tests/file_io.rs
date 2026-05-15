use pad::buffer::Buffer;
use tempfile::tempdir;

#[test]
fn load_existing_file_preserves_lf() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("a.txt");
    std::fs::write(&p, "hello\nworld\n").unwrap();
    let b = Buffer::load_from_file(&p).expect("load");
    assert_eq!(b.text(), "hello\nworld\n");
    assert!(!b.is_dirty());
}

#[test]
fn load_missing_returns_empty_buffer_with_note() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("missing.txt");
    let r = Buffer::load_from_file(&p).expect("load missing");
    assert_eq!(r.text(), "");
    assert!(!r.is_dirty());
}

#[test]
fn save_round_trips_crlf() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("crlf.txt");
    std::fs::write(&p, "a\r\nb\r\n").unwrap();
    let mut b = Buffer::load_from_file(&p).unwrap();
    b.insert_char('X');
    b.save_to_file(&p).unwrap();
    let after = std::fs::read_to_string(&p).unwrap();
    assert_eq!(after, "Xa\r\nb\r\n");
}

#[test]
fn save_marks_clean() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("clean.txt");
    let mut b = Buffer::empty();
    b.insert_str("hi");
    assert!(b.is_dirty());
    b.save_to_file(&p).unwrap();
    assert!(!b.is_dirty());
}

#[test]
fn save_non_utf8_path_errors_gracefully() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("bin.dat");
    std::fs::write(&p, [0xff, 0xfe, 0x00, 0x00]).unwrap();
    let r = Buffer::load_from_file(&p);
    assert!(r.is_err(), "non-UTF-8 should bubble an error");
}
