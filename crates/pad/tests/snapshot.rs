use pad::buffer::Buffer;
use pad::buffer::sidecar::SidecarHandle;
use tempfile::tempdir;

#[test]
fn pre_share_snapshot_writes_file() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut buf = Buffer::empty();
    buf.insert_str("contents to preserve");
    sc.pre_share_snapshot(&buf).expect("snapshot");

    let entries: Vec<_> = std::fs::read_dir(sc.dir())
        .unwrap()
        .filter_map(|r| r.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("pre-share-"))
        .collect();
    assert_eq!(entries.len(), 1);

    let snap_path = entries[0].path();
    let contents = std::fs::read_to_string(&snap_path).unwrap();
    assert_eq!(contents, "contents to preserve");
}

#[test]
fn pre_merge_snapshot_overwrites_existing() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut buf = Buffer::empty();
    buf.insert_str("first");
    sc.pre_merge_snapshot(&buf).unwrap();
    buf.insert_str(" second");
    sc.pre_merge_snapshot(&buf).unwrap();
    let contents = std::fs::read_to_string(sc.dir().join("pre-merge.snapshot")).unwrap();
    assert_eq!(contents, "first second");
}
