use crate::keymap::{KeyAction, key_to_action};
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::StreamExt;
use tokio::sync::mpsc;

/// Pure event-to-action dispatch. `None` means "ignore this event".
///
/// Filters key events to `KeyEventKind::Press` only — on Windows,
/// crossterm emits Press AND Release for every keystroke; processing
/// both doubles each typed character. Unix terminals only emit Press,
/// so the filter is a no-op there. Bracketed-paste arrives as
/// `Event::Paste` regardless of platform. Everything else is dropped.
pub fn event_to_action(evt: Event) -> Option<KeyAction> {
    match evt {
        Event::Key(ke) if ke.kind == KeyEventKind::Press => Some(key_to_action(ke)),
        Event::Paste(s) => Some(KeyAction::InsertText(s)),
        _ => None,
    }
}

/// Spawn a tokio task that reads crossterm events and forwards each
/// KeyAction over an unbounded mpsc channel.
///
/// Mouse capture is intentionally OFF (see `Tui::enter`) so the host
/// terminal keeps its right-click / text-select behaviour; PgUp / PgDn
/// / Ctrl-Y / Ctrl-V cover scrolling from the keyboard.
pub fn spawn_event_task() -> mpsc::UnboundedReceiver<KeyAction> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut stream = EventStream::new();
        while let Some(evt) = stream.next().await {
            let Ok(evt) = evt else { continue };
            let Some(action) = event_to_action(evt) else {
                continue;
            };
            if tx.send(action).is_err() {
                break;
            }
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode, kind: KeyEventKind) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn press_events_dispatch() {
        let action = event_to_action(key(KeyCode::Char('a'), KeyEventKind::Press));
        assert_eq!(action, Some(KeyAction::InsertChar('a')));
    }

    #[test]
    fn release_events_drop() {
        // Regression: Sam hit double-character typing on Windows because
        // crossterm-on-Windows emits BOTH Press and Release for every
        // keystroke. Release must be a no-op.
        let action = event_to_action(key(KeyCode::Char('a'), KeyEventKind::Release));
        assert_eq!(action, None);
    }

    #[test]
    fn repeat_events_drop() {
        // Repeat events fire when a key is held down — letting them
        // through would re-trigger handlers like ^X or M-S beyond the
        // user's intent. Auto-repeat for typing is handled by the OS at
        // the terminal layer (which delivers separate Press events).
        let action = event_to_action(key(KeyCode::Char('a'), KeyEventKind::Repeat));
        assert_eq!(action, None);
    }

    #[test]
    fn paste_events_dispatch_regardless_of_kind() {
        let action = event_to_action(Event::Paste("hello".into()));
        assert_eq!(action, Some(KeyAction::InsertText("hello".into())));
    }
}
