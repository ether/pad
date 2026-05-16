use crate::keymap::{KeyAction, key_to_action};
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use tokio::sync::mpsc;

/// Spawn a tokio task that reads crossterm events and forwards each KeyAction
/// over an unbounded mpsc channel. Returns the receiver.
///
/// Handles both `Event::Key` (typing) and `Event::Paste` (bracketed-paste).
/// Bracketed paste must be enabled on the terminal — see `Tui::enter`.
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
