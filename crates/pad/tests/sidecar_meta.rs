use pad::buffer::sidecar::SidecarHandle;
use tempfile::tempdir;

#[test]
fn new_sidecar_creates_dir_with_buffer_id() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).expect("new");
    assert!(sc.dir().exists(), "sidecar dir must exist");
    assert!(
        sc.dir().join("meta.json").exists(),
        "meta.json must be written"
    );
    assert!(!sc.buffer_id().to_string().is_empty());
}

#[test]
fn meta_round_trips_file_path() {
    let state_root = tempdir().unwrap();
    let file_path = std::path::PathBuf::from("/some/file.md");
    let sc = SidecarHandle::for_file(state_root.path(), &file_path).expect("for_file");
    let id = sc.buffer_id();
    drop(sc);
    let reattached = SidecarHandle::reattach(state_root.path(), &id).expect("reattach");
    assert_eq!(reattached.file_path().unwrap(), file_path);
}

#[test]
fn buffer_id_is_uuid_v7() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let id_str = sc.buffer_id().to_string();
    let uuid = uuid::Uuid::parse_str(&id_str).expect("parses as uuid");
    assert_eq!(uuid.get_version_num(), 7);
}
