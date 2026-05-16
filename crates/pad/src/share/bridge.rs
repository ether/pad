use etherpad_client::changeset::{Changeset, Op, OpCode};

/// Etherpad's canonical form requires `|<lines>` to prefix any op (`=`/`+`/`-`)
/// whose run contains a newline AND that newline must end the run. The trailing-
/// keep "implicit" rule omits a final all-keep but doesn't relax this.
///
/// `count_newlines_in_chars(s, char_count)` returns the number of `\n` in the
/// first `char_count` characters of `s`.
fn count_newlines_in_chars(s: &str, char_count: u32) -> u32 {
    s.chars()
        .take(char_count as usize)
        .filter(|c| *c == '\n')
        .count() as u32
}

/// Build a Changeset that inserts `text` at char offset `pos` in the document
/// `old_text`. Counts newlines in the surrounding keep regions for canonical
/// `|<lines>=<count>` encoding.
pub fn changeset_for_insert(old_text: &str, pos: u32, text: &str) -> Changeset {
    let old_len = old_text.chars().count() as u32;
    let inserted = text.chars().count() as u32;
    let inserted_lines = text.matches('\n').count() as u32;
    let mut ops = Vec::new();
    // Leading keep: chars 0..pos of old_text.
    if pos > 0 {
        let lines = count_newlines_in_chars(old_text, pos);
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: pos,
            lines,
            attribs: vec![],
        });
    }
    // Etherpad's checkRep enforces: an insert op with `|N` (lines > 0) MUST
    // have its char_bank slice end with `\n` (Changeset.ts checkRep:
    // "multiline insert op does not end with a newline"). When the inserted
    // text has newlines but its tail is mid-line (e.g. paste of
    // "A\nB\nC"), split into:
    //   - Insert lines=N covering A\nB\n
    //   - Insert lines=0 covering C
    // both with the same author attribute (added downstream in
    // PadSession::send_changeset).
    if inserted_lines > 0 && !text.ends_with('\n') {
        let last_nl_byte = text.rfind('\n').expect("has \\n");
        let prefix = &text[..last_nl_byte + 1];
        let suffix = &text[last_nl_byte + 1..];
        let prefix_chars = prefix.chars().count() as u32;
        let suffix_chars = suffix.chars().count() as u32;
        ops.push(Op {
            opcode: OpCode::Insert,
            chars: prefix_chars,
            lines: inserted_lines,
            attribs: vec![],
        });
        ops.push(Op {
            opcode: OpCode::Insert,
            chars: suffix_chars,
            lines: 0,
            attribs: vec![],
        });
    } else {
        ops.push(Op {
            opcode: OpCode::Insert,
            chars: inserted,
            lines: inserted_lines,
            attribs: vec![],
        });
    }
    // Trailing keep is implicit in canonical form (final Keep covering rest
    // of doc is omitted).
    Changeset {
        old_len,
        net_delta: inserted as i64,
        ops,
        char_bank: text.to_string(),
    }
}

/// Build a Changeset that deletes `deleted_text` (which lives at offset `pos`
/// in the document `old_text`).
///
/// **Wire-format note:** Etherpad's `checkRep` asserts the `char_bank` is
/// EMPTY at the end of a changeset — only Insert ops consume from it. So
/// despite our knowing what the deleted text is, the bank STAYS empty.
/// The deleted chars are reconstructed server-side from the source text
/// + op.chars (see Changeset.ts `applyToText` case `-`).
pub fn changeset_for_delete(old_text: &str, pos: u32, deleted_text: String) -> Changeset {
    let old_len = old_text.chars().count() as u32;
    let deleted_chars = deleted_text.chars().count() as u32;
    let deleted_lines = deleted_text.matches('\n').count() as u32;
    let mut ops = Vec::new();
    if pos > 0 {
        let lines = count_newlines_in_chars(old_text, pos);
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: pos,
            lines,
            attribs: vec![],
        });
    }
    ops.push(Op {
        opcode: OpCode::Delete,
        chars: deleted_chars,
        lines: deleted_lines,
        attribs: vec![],
    });
    // Trailing keep is implicit — omit even when non-zero.
    Changeset {
        old_len,
        net_delta: -(deleted_chars as i64),
        ops,
        char_bank: String::new(),
    }
}
