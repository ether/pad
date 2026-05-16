use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    InsertChar(char),
    /// Atomic paste — a single block of text that should land in one mutation
    /// + one outbound Changeset. Bracketed-paste comes through this path.
    InsertText(String),
    Backspace,
    DeleteForward,
    Left,
    Right,
    Up,
    Down,
    WriteOut,      // ^O
    Exit,          // ^X
    InsertFile,    // ^R
    Cut,           // ^K
    Uncut,         // ^U
    WhereIs,       // ^W
    Replace,       // M-R
    GotoLine,      // ^_
    Undo,          // M-U
    Redo,          // M-E
    Help,          // ^G
    CursorPos,     // ^C
    Suspend,       // ^Z
    Share,         // M-S
    ToggleAuthors, // M-A
    CopyShareUrl,  // M-C
    ReshowQr,      // M-Q
    Unbound,       // ^S and other dead keys
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
            's' => KeyAction::Share,
            'a' => KeyAction::ToggleAuthors,
            'c' => KeyAction::CopyShareUrl,
            'q' => KeyAction::ReshowQr,
            _ => KeyAction::Unbound,
        },
        (KeyCode::Char(c), false, false) => KeyAction::InsertChar(c),
        (KeyCode::Enter, _, _) => KeyAction::InsertChar('\n'),
        // Tab inserts 4 spaces rather than a literal '\t'. Etherpad has no
        // sensible rendering for raw tabs (its browser uses a `list:indent`
        // line attribute for indent and falls back to visible-marker glyphs
        // for stray '\t' chars — which surfaces as '*'-looking bullets on
        // each indented line). Four spaces matches Etherpad's own
        // THE_TAB = '    ' constant in ace2_inner.ts. Local file editing
        // keeps spaces-not-tabs as a side effect, which matches common
        // editor defaults.
        (KeyCode::Tab, _, _) => KeyAction::InsertText("    ".to_string()),
        (KeyCode::Backspace, _, _) => KeyAction::Backspace,
        (KeyCode::Delete, _, _) => KeyAction::DeleteForward,
        (KeyCode::Left, _, _) => KeyAction::Left,
        (KeyCode::Right, _, _) => KeyAction::Right,
        (KeyCode::Up, _, _) => KeyAction::Up,
        (KeyCode::Down, _, _) => KeyAction::Down,
        _ => KeyAction::Unbound,
    }
}
