use expectrl::{Eof, Expect, spawn};
use std::time::Duration;
use tempfile::tempdir;

/// Regression: pressing ^C (cursor-pos flash) then ^X used to swallow the ^X
/// because the FlashMessage handler returned without re-dispatching the key.
#[test]
fn cursor_pos_flash_then_exit() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("flash.txt");
    std::fs::write(&path, "").unwrap();
    let bin = env!("CARGO_BIN_EXE_pad");
    let mut p = spawn(format!("{} {}", bin, path.display())).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    std::thread::sleep(Duration::from_millis(300));
    // Buffer is clean — pressing ^C flashes cursor pos, then ^X should exit.
    p.send([0x03u8].as_slice()).unwrap(); // ^C
    std::thread::sleep(Duration::from_millis(150));
    p.send([0x18u8].as_slice()).unwrap(); // ^X

    p.expect(Eof).expect("editor must exit after ^C ^X");
}

/// Regression: ^G (help overlay) → ^X used to swallow the ^X for the same reason.
#[test]
fn help_overlay_then_exit() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("help.txt");
    std::fs::write(&path, "").unwrap();
    let bin = env!("CARGO_BIN_EXE_pad");
    let mut p = spawn(format!("{} {}", bin, path.display())).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    std::thread::sleep(Duration::from_millis(300));
    p.send([0x07u8].as_slice()).unwrap(); // ^G (BEL)
    std::thread::sleep(Duration::from_millis(150));
    p.send([0x18u8].as_slice()).unwrap(); // ^X

    p.expect(Eof).expect("editor must exit after ^G ^X");
}

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
