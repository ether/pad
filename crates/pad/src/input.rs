use crate::keymap::{KeyAction, key_to_action};
use crossterm::event::{Event, EventStream, MouseEvent, MouseEventKind};
use futures_util::StreamExt;
use tokio::sync::mpsc;

/// Spawn a tokio task that reads crossterm events and forwards each KeyAction
/// over an unbounded mpsc channel. Returns the receiver.
///
/// Handles `Event::Key` (typing), `Event::Paste` (bracketed-paste), and
/// `Event::Mouse` (scroll-wheel → Up/Down so the caret follows the wheel).
/// Mouse capture must be enabled on the terminal — see `Tui::enter`.
pub fn spawn_event_task() -> mpsc::UnboundedReceiver<KeyAction> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut stream = EventStream::new();
        while let Some(evt) = stream.next().await {
            let actions: Vec<KeyAction> = match evt {
                Ok(Event::Key(ke)) => vec![key_to_action(ke)],
                Ok(Event::Paste(s)) => vec![KeyAction::InsertText(s)],
                Ok(Event::Mouse(MouseEvent { kind, .. })) => match kind {
                    // Scroll wheel translates to cursor-up / cursor-down by
                    // 3 lines per tick. We bind to cursor moves (not pure
                    // viewport scroll) so the caret always stays where the
                    // user can see it — viewport auto-scrolls to follow.
                    MouseEventKind::ScrollUp => vec![
                        KeyAction::Up,
                        KeyAction::Up,
                        KeyAction::Up,
                    ],
                    MouseEventKind::ScrollDown => vec![
                        KeyAction::Down,
                        KeyAction::Down,
                        KeyAction::Down,
                    ],
                    _ => continue,
                },
                _ => continue,
            };
            for action in actions {
                if tx.send(action).is_err() {
                    return;
                }
            }
        }
    });
    rx
}
