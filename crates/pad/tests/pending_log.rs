use pad::buffer::sidecar::{PendingEntry, PendingLog, SidecarHandle};
use tempfile::tempdir;

#[test]
fn append_then_read_back() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert {
        offset: 0,
        text: "hi".into(),
    })
    .unwrap();
    log.append(&PendingEntry::Delete { offset: 1, len: 1 }).unwrap();
    drop(log);
    let entries = PendingLog::read_all(&sc).unwrap();
    assert_eq!(entries.len(), 2);
    match &entries[0] {
        PendingEntry::Insert { offset, text } => {
            assert_eq!(*offset, 0);
            assert_eq!(text, "hi");
        }
        e => panic!("expected Insert, got {e:?}"),
    }
}

#[test]
fn truncate_on_save_clears_log() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert {
        offset: 0,
        text: "x".into(),
    })
    .unwrap();
    log.truncate().unwrap();
    let entries = PendingLog::read_all(&sc).unwrap();
    assert!(entries.is_empty(), "log should be empty after truncate");
}

#[test]
fn ignores_corrupt_trailing_entry() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    {
        let mut log = PendingLog::open(&sc).unwrap();
        log.append(&PendingEntry::Insert {
            offset: 0,
            text: "hi".into(),
        })
        .unwrap();
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(sc.pending_log_path())
        .unwrap();
    f.write_all(b"{garbage no newline").unwrap();

    let entries = PendingLog::read_all(&sc).unwrap();
    assert_eq!(entries.len(), 1, "corrupt tail must be ignored");
}
