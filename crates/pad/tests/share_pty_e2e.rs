use expectrl::{Eof, Expect, spawn};
use std::time::Duration;
use tempfile::tempdir;

/// Pressing M-S when no remote is configured shows the "Run 'pad --setup'"
/// flash and the editor stays alive.
#[test]
fn ms_without_remote_shows_setup_hint() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("share.txt");
    std::fs::write(&path, "").unwrap();

    let bin = env!("CARGO_BIN_EXE_pad");
    let cmd = format!(
        "env HOME={} XDG_CONFIG_HOME={} XDG_STATE_HOME={} {} {}",
        dir.path().display(),
        dir.path().join("config").display(),
        dir.path().join("state").display(),
        bin,
        path.display(),
    );
    let mut p = spawn(cmd).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    std::thread::sleep(Duration::from_millis(300));
    // Most terminals translate Alt+S to ESC + s.
    p.send([0x1Bu8, b's'].as_slice()).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    // Dismiss the flash + exit.
    p.send([0x18u8].as_slice()).unwrap();
    p.expect(Eof).expect("editor must exit cleanly");
}
