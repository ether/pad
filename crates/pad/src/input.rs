use crate::keymap::{KeyAction, key_to_action};
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use tokio::sync::mpsc;

/// Spawn a tokio task that reads crossterm events and forwards each
/// KeyAction over an unbounded mpsc channel.
///
/// Handles `Event::Key` (typing) and `Event::Paste` (bracketed-paste).
/// Mouse capture is intentionally OFF (see `Tui::enter`) so the host
/// terminal keeps its right-click / text-select behaviour; PgUp / PgDn
/// / Ctrl-Y / Ctrl-V cover scrolling from the keyboard.
pub fn spawn_event_task() -> mpsc::UnboundedReceiver<KeyAction> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut stream = EventStream::new();
        while let Some(evt) = stream.next().await {
            let action = match evt {
                Ok(Event::Key(ke)) => key_to_action(ke),
                Ok(Event::Paste(s)) => KeyAction::InsertText(s),
                _ => continue,
            };
            if tx.send(action).is_err() {
                break;
            }
        }
    });
    rx
}
