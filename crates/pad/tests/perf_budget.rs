use pad::buffer::Buffer;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn cold_open_100kb_under_50ms() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("100k.txt");
    std::fs::write(&path, "x".repeat(100 * 1024)).unwrap();

    let _ = Buffer::load_from_file(&path).unwrap();

    let mut samples = Vec::new();
    for _ in 0..20 {
        let t = Instant::now();
        let _b = Buffer::load_from_file(&path).unwrap();
        samples.push(t.elapsed());
    }
    samples.sort();
    let p50 = samples[samples.len() / 2];
    let p99 = samples[(samples.len() * 99) / 100];
    eprintln!("cold_open_100kb p50={p50:?} p99={p99:?}");
    assert!(
        p99 < std::time::Duration::from_millis(50),
        "p99 cold open exceeded 50 ms: {p99:?}",
    );
}

#[test]
fn empty_buffer_under_1ms() {
    let mut samples = Vec::new();
    for _ in 0..100 {
        let t = Instant::now();
        let _b = Buffer::empty();
        samples.push(t.elapsed());
    }
    samples.sort();
    let p99 = samples[(samples.len() * 99) / 100];
    assert!(
        p99 < std::time::Duration::from_millis(1),
        "Buffer::empty too slow: {p99:?}"
    );
}
