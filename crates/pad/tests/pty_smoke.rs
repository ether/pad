use expectrl::{Eof, Expect, spawn};
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn open_edit_save_exit_cycle() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("hello.txt");
    let bin = env!("CARGO_BIN_EXE_pad");
    let mut p = spawn(format!("{} {}", bin, path.display())).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    // Allow the editor to enter raw mode + draw the first frame.
    std::thread::sleep(Duration::from_millis(300));

    // Type "Hi"
    p.send("Hi").unwrap();
    std::thread::sleep(Duration::from_millis(100));

    // Save with ^O (0x0F = SI = Ctrl+O)
    p.send([0x0Fu8].as_slice()).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    // Exit with ^X (0x18 = CAN = Ctrl+X)
    p.send([0x18u8].as_slice()).unwrap();

    p.expect(Eof).expect("editor must exit");

    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        saved.contains("Hi"),
        "saved file must contain 'Hi', got {saved:?}"
    );
}
