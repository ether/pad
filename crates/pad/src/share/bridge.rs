use etherpad_client::changeset::{Changeset, Op, OpCode};

/// Push the op(s) covering `run` (the exact chars a keep/delete/insert spans)
/// onto `ops`, in Etherpad's canonical form.
///
/// Etherpad's `checkRep` requires any op with `|<lines>` (lines > 0) to end its
/// run on a `\n`. Browsers apply changesets line-by-line (`mutateTextLines`),
/// so a multi-line op that ends mid-line makes them drop or misplace chars.
/// When `run` has newlines but doesn't end with one, split it at the last
/// `\n` into `|N<op><chars through last \n>` followed by `<op><rest>` with
/// lines 0. E.g. a keep over `"ab\ncd\nef"` becomes `|2=6=2`, not `|2=8`.
///
/// Attributes are left empty; callers add authorship downstream
/// (see `PadSession::send_changeset`).
fn push_run(ops: &mut Vec<Op>, opcode: OpCode, run: &str) {
    let chars = run.chars().count() as u32;
    if chars == 0 {
        return;
    }
    let lines = run.matches('\n').count() as u32;
    if lines > 0 && !run.ends_with('\n') {
        let last_nl_byte = run.rfind('\n').expect("has \\n");
        let prefix_chars = run[..last_nl_byte + 1].chars().count() as u32;
        ops.push(Op {
            opcode,
            chars: prefix_chars,
            lines,
            attribs: vec![],
        });
        ops.push(Op {
            opcode,
            chars: chars - prefix_chars,
            lines: 0,
            attribs: vec![],
        });
    } else {
        ops.push(Op {
            opcode,
            chars,
            lines,
            attribs: vec![],
        });
    }
}

/// The first `char_count` characters of `s`.
fn char_prefix(s: &str, char_count: u32) -> &str {
    match s.char_indices().nth(char_count as usize) {
        Some((byte, _)) => &s[..byte],
        None => s,
    }
}

/// Build a Changeset that inserts `text` at char offset `pos` in the document
/// `old_text`. The leading keep and the insert are both split at line
/// boundaries as needed (see [`push_run`]).
pub fn changeset_for_insert(old_text: &str, pos: u32, text: &str) -> Changeset {
    let old_len = old_text.chars().count() as u32;
    let inserted = text.chars().count() as u32;
    let mut ops = Vec::new();
    push_run(&mut ops, OpCode::Keep, char_prefix(old_text, pos));
    push_run(&mut ops, OpCode::Insert, text);
    // Trailing keep is implicit in canonical form (final Keep covering rest
    // of doc is omitted).
    Changeset {
        old_len,
        net_delta: inserted as i64,
        ops,
        char_bank: text.to_string(),
    }
}

/// Build a Changeset that replaces `deleted_text` at offset `pos` in
/// `old_text` with `inserted_text` — a single-changeset form for the
/// Replace path so the server applies and broadcasts the substitution as
/// one atomic step. Keep, delete and insert runs are each split at line
/// boundaries as needed (see [`push_run`]).
pub fn changeset_for_replace(
    old_text: &str,
    pos: u32,
    deleted_text: &str,
    inserted_text: &str,
) -> Changeset {
    let old_len = old_text.chars().count() as u32;
    let deleted_chars = deleted_text.chars().count() as u32;
    let inserted_chars = inserted_text.chars().count() as u32;
    let mut ops = Vec::new();
    push_run(&mut ops, OpCode::Keep, char_prefix(old_text, pos));
    push_run(&mut ops, OpCode::Delete, deleted_text);
    push_run(&mut ops, OpCode::Insert, inserted_text);
    Changeset {
        old_len,
        net_delta: inserted_chars as i64 - deleted_chars as i64,
        ops,
        char_bank: inserted_text.to_string(),
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
    let mut ops = Vec::new();
    push_run(&mut ops, OpCode::Keep, char_prefix(old_text, pos));
    push_run(&mut ops, OpCode::Delete, &deleted_text);
    // Trailing keep is implicit — omit even when non-zero.
    Changeset {
        old_len,
        net_delta: -(deleted_chars as i64),
        ops,
        char_bank: String::new(),
    }
}
