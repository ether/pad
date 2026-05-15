use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pad::buffer::Buffer;
use std::path::PathBuf;
use tempfile::tempdir;

fn bench_open_empty(c: &mut Criterion) {
    c.bench_function("Buffer::empty()", |b| {
        b.iter(|| black_box(Buffer::empty()));
    });
}

fn bench_load_1k(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let path: PathBuf = dir.path().join("1k.txt");
    std::fs::write(&path, "x".repeat(1024)).unwrap();
    c.bench_function("Buffer::load_from_file (1KB)", |b| {
        b.iter(|| black_box(Buffer::load_from_file(&path).unwrap()));
    });
}

fn bench_load_100k(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let path: PathBuf = dir.path().join("100k.txt");
    std::fs::write(&path, "x".repeat(100 * 1024)).unwrap();
    c.bench_function("Buffer::load_from_file (100KB)", |b| {
        b.iter(|| black_box(Buffer::load_from_file(&path).unwrap()));
    });
}

criterion_group!(benches, bench_open_empty, bench_load_1k, bench_load_100k);
criterion_main!(benches);
