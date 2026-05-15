use pad::buffer::sidecar::{PendingEntry, PendingLog, SidecarHandle};
use pad::recover::{list_recoverable, replay_into_buffer};
use tempfile::tempdir;

#[test]
fn list_recoverable_finds_buffer_with_log_entries() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert {
        offset: 0,
        text: "hi".into(),
    })
    .unwrap();

    let mut found = list_recoverable(state_root.path()).unwrap();
    found.retain(|r| r.buffer_id == sc.buffer_id());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].entry_count, 1);
}

#[test]
fn replay_into_buffer_reconstructs_text() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert {
        offset: 0,
        text: "hello".into(),
    })
    .unwrap();
    log.append(&PendingEntry::Delete { offset: 4, len: 1 })
        .unwrap();

    let buf = replay_into_buffer(&sc).unwrap();
    assert_eq!(buf.text(), "hell");
}
