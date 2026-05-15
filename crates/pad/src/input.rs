use crate::keymap::{KeyAction, key_to_action};
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use tokio::sync::mpsc;

/// Spawn a tokio task that reads crossterm events and forwards each KeyAction
/// over an unbounded mpsc channel. Returns the receiver.
pub fn spawn_event_task() -> mpsc::UnboundedReceiver<KeyAction> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut stream = EventStream::new();
        while let Some(evt) = stream.next().await {
            let Ok(Event::Key(ke)) = evt else { continue };
            if tx.send(key_to_action(ke)).is_err() {
                break;
            }
        }
    });
    rx
}
