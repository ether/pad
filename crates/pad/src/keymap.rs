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
    /// Home / ^A — start of current line.
    LineStart,
    /// End / ^E — end of current line.
    LineEnd,
    /// M-\\ or M-< or Ctrl-Home — top of document.
    DocumentStart,
    /// M-/ or M-> or Ctrl-End — end of document content.
    DocumentEnd,
    /// PageUp / ^Y — scroll viewport up one screen-ish (~half page).
    PageUp,
    /// PageDown / ^V — scroll viewport down one screen-ish (~half page).
    PageDown,
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
            // Ctrl-\ (and its synonym Ctrl-|) is nano's canonical Replace
            // binding. Char comes through as '\\' on most terminals.
            '\\' | '|' => KeyAction::Replace,
            // Nano-canonical emacs-style cursor + edit alternatives. Users
            // who came from nano (or readline) have these in muscle memory:
            //   ^A / ^E — beginning / end of line
            //   ^B / ^F — back / forward one cell  (= Left / Right)
            //   ^P / ^N — previous / next line     (= Up / Down)
            //   ^D       — delete forward          (= Delete key)
            //   ^H       — backspace                (= Backspace key)
            'a' => KeyAction::LineStart,
            'e' => KeyAction::LineEnd,
            'b' => KeyAction::Left,
            'f' => KeyAction::Right,
            'p' => KeyAction::Up,
            'n' => KeyAction::Down,
            'd' => KeyAction::DeleteForward,
            'h' => KeyAction::Backspace,
            // Nano's canonical paging: ^Y = previous, ^V = next.
            'y' => KeyAction::PageUp,
            'v' => KeyAction::PageDown,
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
            'g' => KeyAction::GotoLine,
            // M-, and M-< — jump to top of document. M-. and M-> — jump
            // to end. Nano's canonical pair; the angle-bracket variants
            // are easier on keyboards where comma/period are unshifted.
            ',' | '<' => KeyAction::DocumentStart,
            '.' | '>' => KeyAction::DocumentEnd,
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
        (KeyCode::Home, true, _) => KeyAction::DocumentStart,
        (KeyCode::Home, _, _) => KeyAction::LineStart,
        (KeyCode::End, true, _) => KeyAction::DocumentEnd,
        (KeyCode::End, _, _) => KeyAction::LineEnd,
        (KeyCode::PageUp, _, _) => KeyAction::PageUp,
        (KeyCode::PageDown, _, _) => KeyAction::PageDown,
        _ => KeyAction::Unbound,
    }
}
