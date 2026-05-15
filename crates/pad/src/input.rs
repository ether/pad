use crate::keymap::{KeyAction, key_to_action};
use crossterm::event::{self, Event};
use std::time::Duration;

/// Block on the next key event for up to `timeout`. Returns None on timeout.
pub fn next_action(timeout: Duration) -> anyhow::Result<Option<KeyAction>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    match event::read()? {
        Event::Key(ev) => Ok(Some(key_to_action(ev))),
        _ => Ok(None),
    }
}
