use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pad::keymap::{KeyAction, key_to_action};

fn k(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}
fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}
fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

#[test]
fn plain_char_inserts() {
    assert_eq!(key_to_action(k('a')), KeyAction::InsertChar('a'));
}
#[test]
fn ctrl_o_is_write_out() {
    assert_eq!(key_to_action(ctrl('o')), KeyAction::WriteOut);
}
#[test]
fn ctrl_x_is_exit() {
    assert_eq!(key_to_action(ctrl('x')), KeyAction::Exit);
}
#[test]
fn ctrl_k_is_cut() {
    assert_eq!(key_to_action(ctrl('k')), KeyAction::Cut);
}
#[test]
fn ctrl_u_is_uncut() {
    assert_eq!(key_to_action(ctrl('u')), KeyAction::Uncut);
}
#[test]
fn ctrl_w_is_where_is() {
    assert_eq!(key_to_action(ctrl('w')), KeyAction::WhereIs);
}
#[test]
fn ctrl_underscore_is_goto_line() {
    assert_eq!(key_to_action(ctrl('_')), KeyAction::GotoLine);
}
#[test]
fn alt_u_is_undo() {
    assert_eq!(key_to_action(alt('u')), KeyAction::Undo);
}
#[test]
fn alt_e_is_redo() {
    assert_eq!(key_to_action(alt('e')), KeyAction::Redo);
}
#[test]
fn alt_r_is_replace() {
    assert_eq!(key_to_action(alt('r')), KeyAction::Replace);
}
#[test]
fn ctrl_g_is_help() {
    assert_eq!(key_to_action(ctrl('g')), KeyAction::Help);
}
#[test]
fn ctrl_c_is_cursor_pos() {
    assert_eq!(key_to_action(ctrl('c')), KeyAction::CursorPos);
}
#[test]
fn ctrl_z_is_suspend() {
    assert_eq!(key_to_action(ctrl('z')), KeyAction::Suspend);
}
#[test]
fn ctrl_r_is_insert_file() {
    assert_eq!(key_to_action(ctrl('r')), KeyAction::InsertFile);
}
#[test]
fn backspace() {
    let e = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    assert_eq!(key_to_action(e), KeyAction::Backspace);
}
#[test]
fn arrows() {
    let l = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    let r = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    let u = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    let d = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(key_to_action(l), KeyAction::Left);
    assert_eq!(key_to_action(r), KeyAction::Right);
    assert_eq!(key_to_action(u), KeyAction::Up);
    assert_eq!(key_to_action(d), KeyAction::Down);
}
#[test]
fn enter_inserts_newline() {
    let e = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(key_to_action(e), KeyAction::InsertChar('\n'));
}
#[test]
fn ctrl_s_is_unbound() {
    assert_eq!(key_to_action(ctrl('s')), KeyAction::Unbound);
}
