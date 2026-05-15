use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    InsertChar(char),
    Backspace,
    DeleteForward,
    Left,
    Right,
    Up,
    Down,
    WriteOut,   // ^O
    Exit,       // ^X
    InsertFile, // ^R
    Cut,        // ^K
    Uncut,      // ^U
    WhereIs,    // ^W
    Replace,    // M-R
    GotoLine,   // ^_
    Undo,       // M-U
    Redo,       // M-E
    Help,       // ^G
    CursorPos,  // ^C
    Suspend,    // ^Z
    Unbound,    // ^S and other dead keys
}

pub fn key_to_action(ev: KeyEvent) -> KeyAction {
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    let alt = ev.modifiers.contains(KeyModifiers::ALT);
    match (ev.code, ctrl, alt) {
        (KeyCode::Char(c), true, _) => match c.to_ascii_lowercase() {
            'o' => KeyAction::WriteOut,
            'x' => KeyAction::Exit,
            'r' => KeyAction::InsertFile,
            'k' => KeyAction::Cut,
            'u' => KeyAction::Uncut,
            'w' => KeyAction::WhereIs,
            '_' | '/' => KeyAction::GotoLine,
            'g' => KeyAction::Help,
            'c' => KeyAction::CursorPos,
            'z' => KeyAction::Suspend,
            's' => KeyAction::Unbound,
            _ => KeyAction::Unbound,
        },
        (KeyCode::Char(c), _, true) => match c.to_ascii_lowercase() {
            'r' => KeyAction::Replace,
            'u' => KeyAction::Undo,
            'e' => KeyAction::Redo,
            _ => KeyAction::Unbound,
        },
        (KeyCode::Char(c), false, false) => KeyAction::InsertChar(c),
        (KeyCode::Enter, _, _) => KeyAction::InsertChar('\n'),
        (KeyCode::Tab, _, _) => KeyAction::InsertChar('\t'),
        (KeyCode::Backspace, _, _) => KeyAction::Backspace,
        (KeyCode::Delete, _, _) => KeyAction::DeleteForward,
        (KeyCode::Left, _, _) => KeyAction::Left,
        (KeyCode::Right, _, _) => KeyAction::Right,
        (KeyCode::Up, _, _) => KeyAction::Up,
        (KeyCode::Down, _, _) => KeyAction::Down,
        _ => KeyAction::Unbound,
    }
}
