use etherpad_client::changeset::{Changeset, Op, OpCode};

/// Build a Changeset that inserts `text` at char offset `pos` in a document of
/// length `old_len` chars.
pub fn changeset_for_insert(old_len: u32, pos: u32, text: &str) -> Changeset {
    let inserted = text.chars().count() as u32;
    let mut ops = Vec::new();
    if pos > 0 {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: pos,
            lines: 0,
            attribs: vec![],
        });
    }
    ops.push(Op {
        opcode: OpCode::Insert,
        chars: inserted,
        lines: text.matches('\n').count() as u32,
        attribs: vec![],
    });
    if pos < old_len {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: old_len - pos,
            lines: 0,
            attribs: vec![],
        });
    }
    Changeset {
        old_len,
        net_delta: inserted as i64,
        ops,
        char_bank: text.to_string(),
    }
}

/// Build a Changeset that deletes `deleted_text` (which lives at offset `pos`
/// in a document of length `old_len`).
pub fn changeset_for_delete(old_len: u32, pos: u32, deleted_text: String) -> Changeset {
    let deleted_chars = deleted_text.chars().count() as u32;
    let mut ops = Vec::new();
    if pos > 0 {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: pos,
            lines: 0,
            attribs: vec![],
        });
    }
    ops.push(Op {
        opcode: OpCode::Delete,
        chars: deleted_chars,
        lines: deleted_text.matches('\n').count() as u32,
        attribs: vec![],
    });
    let after = old_len - pos - deleted_chars;
    if after > 0 {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: after,
            lines: 0,
            attribs: vec![],
        });
    }
    Changeset {
        old_len,
        net_delta: -(deleted_chars as i64),
        ops,
        char_bank: deleted_text,
    }
}
