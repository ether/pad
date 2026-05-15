use crate::buffer::{Buffer, CursorPos};
use crate::share::outbound::OutboundQueue;
use etherpad_client::changeset::{Changeset, OpCode};
use etherpad_client::ot;

/// Apply a remote changeset to the local buffer, after OT-rebasing it against
/// any unACK'd local changesets in `queue`. Also remaps the local cursor so it
/// stays where the user intended after remote inserts/deletes shifted things.
pub fn apply_remote(
    buffer: &mut Buffer,
    remote: &Changeset,
    queue: &OutboundQueue,
) -> anyhow::Result<()> {
    let mut rebased = remote.clone();
    for local in queue.pending() {
        rebased = ot::follow(local, &rebased).map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    let old_offset = buffer.cursor_offset();
    let new_offset = remap_offset(old_offset, &rebased);

    let before = buffer.text();
    let after = ot::apply(&rebased, &before).map_err(|e| anyhow::anyhow!("{e}"))?;

    buffer.replace_all_text(&after);
    let new_pos = offset_to_cursor_pos(buffer, new_offset);
    buffer.move_cursor_to(new_pos);
    Ok(())
}

fn remap_offset(offset: u32, cs: &Changeset) -> u32 {
    let mut consumed_output = 0u32;
    let mut remaining = offset;
    for op in &cs.ops {
        let n = op.chars;
        match op.opcode {
            OpCode::Keep => {
                let take = n.min(remaining);
                consumed_output += take;
                remaining -= take;
                if remaining == 0 {
                    return consumed_output;
                }
            }
            OpCode::Insert => {
                // Inserts that appear in the op stream BEFORE we've finished
                // walking past the cursor's original position are inserts
                // that land in front of the cursor.
                consumed_output += n;
            }
            OpCode::Delete => {
                let take = n.min(remaining);
                remaining -= take;
                if remaining == 0 {
                    return consumed_output;
                }
            }
        }
    }
    consumed_output + remaining
}

fn offset_to_cursor_pos(buffer: &Buffer, offset: u32) -> CursorPos {
    let mut remaining = offset as usize;
    for li in 0..buffer.line_count() {
        let line = buffer.line(li);
        let line_chars = line.chars().count() + 1;
        if remaining < line_chars {
            return CursorPos {
                line: li,
                col: remaining.min(line.chars().count()),
            };
        }
        remaining -= line_chars;
    }
    CursorPos {
        line: buffer.line_count() - 1,
        col: buffer.line(buffer.line_count() - 1).chars().count(),
    }
}
