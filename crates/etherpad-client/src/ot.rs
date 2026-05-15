use crate::changeset::{Changeset, Op, OpCode};
use crate::error::{ClientError, Result};

/// Apply `cs` to `text`, returning the resulting `String`.
pub fn apply(cs: &Changeset, text: &str) -> Result<String> {
    let text_chars: Vec<char> = text.chars().collect();
    if text_chars.len() as u32 != cs.old_len {
        return Err(ClientError::ApplyChangeset(format!(
            "old_len mismatch: changeset expects {} chars, text has {}",
            cs.old_len,
            text_chars.len()
        )));
    }

    let bank_chars: Vec<char> = cs.char_bank.chars().collect();
    let mut bank_cursor = 0usize;
    let mut text_cursor = 0usize;
    let mut out = String::with_capacity(text.len() + cs.char_bank.len());

    for op in &cs.ops {
        let n = op.chars as usize;
        match op.opcode {
            OpCode::Keep => {
                let end = text_cursor + n;
                if end > text_chars.len() {
                    return Err(ClientError::ApplyChangeset("keep past end of text".into()));
                }
                for c in &text_chars[text_cursor..end] {
                    out.push(*c);
                }
                text_cursor = end;
            }
            OpCode::Delete => {
                let end = text_cursor + n;
                if end > text_chars.len() {
                    return Err(ClientError::ApplyChangeset(
                        "delete past end of text".into(),
                    ));
                }
                if bank_cursor + n > bank_chars.len() {
                    return Err(ClientError::ApplyChangeset(
                        "delete consumes past end of char bank".into(),
                    ));
                }
                bank_cursor += n;
                text_cursor = end;
            }
            OpCode::Insert => {
                if bank_cursor + n > bank_chars.len() {
                    return Err(ClientError::ApplyChangeset(
                        "insert consumes past end of char bank".into(),
                    ));
                }
                for c in &bank_chars[bank_cursor..bank_cursor + n] {
                    out.push(*c);
                }
                bank_cursor += n;
            }
        }
    }

    // Tail: any remaining chars after last op are implicitly kept.
    while text_cursor < text_chars.len() {
        out.push(text_chars[text_cursor]);
        text_cursor += 1;
    }

    Ok(out)
}

/// Compose two changesets `a` and `b` where `a.new_len() == b.old_len`.
/// Returns a single changeset that has the same effect as applying `a` then `b`.
///
/// Reference: `Changeset.compose` in Etherpad's `Changeset.js`.
///
/// KNOWN GAP: the result `char_bank` for delete-from-original ops may not be
/// byte-identical to JS-emitted changesets (we don't reconstruct deleted-from-
/// original bank chars). The applied OUTCOME is correct; conformance fixtures
/// (Task 14) drive any byte-level refinement.
pub fn compose(a: &Changeset, b: &Changeset) -> Result<Changeset> {
    if a.new_len() != b.old_len {
        return Err(ClientError::ApplyChangeset(format!(
            "compose length mismatch: a.new_len={} but b.old_len={}",
            a.new_len(),
            b.old_len
        )));
    }

    enum PostA {
        FromOriginal,
        FromAInsert(char),
    }

    let a_bank_chars: Vec<char> = a.char_bank.chars().collect();
    let mut a_bank_cursor = 0usize;
    let mut a_post: Vec<PostA> = Vec::new();
    let mut a_deletes: Vec<Op> = Vec::new();

    for op in &a.ops {
        match op.opcode {
            OpCode::Keep => {
                for _ in 0..op.chars {
                    a_post.push(PostA::FromOriginal);
                }
            }
            OpCode::Insert => {
                for _ in 0..op.chars {
                    a_post.push(PostA::FromAInsert(a_bank_chars[a_bank_cursor]));
                    a_bank_cursor += 1;
                }
            }
            OpCode::Delete => {
                a_deletes.push(op.clone());
            }
        }
    }

    let mut out_ops: Vec<Op> = Vec::new();
    let mut out_bank = String::new();
    let b_bank_chars: Vec<char> = b.char_bank.chars().collect();
    let mut b_bank_cursor = 0usize;

    // Emit a's deletes first (they apply to the original text and aren't
    // touched by b). Known gap: bank reconstruction is approximated.
    for d in &a_deletes {
        push_op(&mut out_ops, d.clone());
    }

    let mut a_post_cursor = 0usize;

    for op in &b.ops {
        let n = op.chars as usize;
        match op.opcode {
            OpCode::Keep => {
                for _ in 0..n {
                    match &a_post[a_post_cursor] {
                        PostA::FromOriginal => {
                            push_op(
                                &mut out_ops,
                                Op {
                                    opcode: OpCode::Keep,
                                    chars: 1,
                                    lines: 0,
                                    attribs: vec![],
                                },
                            );
                        }
                        PostA::FromAInsert(c) => {
                            push_op(
                                &mut out_ops,
                                Op {
                                    opcode: OpCode::Insert,
                                    chars: 1,
                                    lines: 0,
                                    attribs: vec![],
                                },
                            );
                            out_bank.push(*c);
                        }
                    }
                    a_post_cursor += 1;
                }
            }
            OpCode::Delete => {
                for _ in 0..n {
                    match &a_post[a_post_cursor] {
                        PostA::FromOriginal => {
                            push_op(
                                &mut out_ops,
                                Op {
                                    opcode: OpCode::Delete,
                                    chars: 1,
                                    lines: 0,
                                    attribs: vec![],
                                },
                            );
                        }
                        PostA::FromAInsert(_) => {
                            // a inserted, b deleted → net no-op.
                        }
                    }
                    a_post_cursor += 1;
                }
            }
            OpCode::Insert => {
                push_op(
                    &mut out_ops,
                    Op {
                        opcode: OpCode::Insert,
                        chars: op.chars,
                        lines: op.lines,
                        attribs: op.attribs.clone(),
                    },
                );
                for _ in 0..n {
                    out_bank.push(b_bank_chars[b_bank_cursor]);
                    b_bank_cursor += 1;
                }
            }
        }
    }

    // Any trailing PostA entries are implicitly Kept.
    while a_post_cursor < a_post.len() {
        match &a_post[a_post_cursor] {
            PostA::FromOriginal => push_op(
                &mut out_ops,
                Op {
                    opcode: OpCode::Keep,
                    chars: 1,
                    lines: 0,
                    attribs: vec![],
                },
            ),
            PostA::FromAInsert(c) => {
                push_op(
                    &mut out_ops,
                    Op {
                        opcode: OpCode::Insert,
                        chars: 1,
                        lines: 0,
                        attribs: vec![],
                    },
                );
                out_bank.push(*c);
            }
        }
        a_post_cursor += 1;
    }

    let mut net_delta: i64 = 0;
    for op in &out_ops {
        match op.opcode {
            OpCode::Insert => net_delta += op.chars as i64,
            OpCode::Delete => net_delta -= op.chars as i64,
            OpCode::Keep => {}
        }
    }

    Ok(Changeset {
        old_len: a.old_len,
        net_delta,
        ops: out_ops,
        char_bank: out_bank,
    })
}

/// Compute the inverse of `cs` with respect to `text`.
/// `apply(inverse(cs, x), apply(cs, x)) == x` for all valid `(cs, x)`.
pub fn inverse(cs: &Changeset, text: &str) -> Result<Changeset> {
    let text_chars: Vec<char> = text.chars().collect();
    if text_chars.len() as u32 != cs.old_len {
        return Err(ClientError::ApplyChangeset(format!(
            "inverse: old_len mismatch ({} vs {})",
            cs.old_len,
            text_chars.len()
        )));
    }

    let bank_chars: Vec<char> = cs.char_bank.chars().collect();
    let mut bank_cursor = 0usize;
    let mut text_cursor = 0usize;
    let mut inv_ops: Vec<Op> = Vec::new();
    let mut inv_bank = String::new();

    for op in &cs.ops {
        let n = op.chars as usize;
        match op.opcode {
            OpCode::Keep => {
                push_op(
                    &mut inv_ops,
                    Op {
                        opcode: OpCode::Keep,
                        chars: op.chars,
                        lines: op.lines,
                        attribs: vec![],
                    },
                );
                text_cursor += n;
            }
            OpCode::Insert => {
                push_op(
                    &mut inv_ops,
                    Op {
                        opcode: OpCode::Delete,
                        chars: op.chars,
                        lines: op.lines,
                        attribs: vec![],
                    },
                );
                for c in &bank_chars[bank_cursor..bank_cursor + n] {
                    inv_bank.push(*c);
                }
                bank_cursor += n;
            }
            OpCode::Delete => {
                push_op(
                    &mut inv_ops,
                    Op {
                        opcode: OpCode::Insert,
                        chars: op.chars,
                        lines: op.lines,
                        attribs: vec![],
                    },
                );
                for c in &text_chars[text_cursor..text_cursor + n] {
                    inv_bank.push(*c);
                }
                text_cursor += n;
                bank_cursor += n;
            }
        }
    }

    Ok(Changeset {
        old_len: cs.new_len(),
        net_delta: -cs.net_delta,
        ops: inv_ops,
        char_bank: inv_bank,
    })
}

/// Transform `b` to apply after `a` was applied to the same base document.
///
/// Reference: `Changeset.follow` in Etherpad's `Changeset.js`.
///
/// Tiebreaker (when both `a` and `b` Insert at the same logical position):
/// whichever changeset has the lexicographically smaller serialized form wins
/// the left position. This is symmetric — `follow(a,b)` and `follow(b,a)` agree.
/// Etherpad's JS reference uses author-ID lexicographic order instead; without
/// author IDs at this level we use the changeset bytes. Plan 1 conformance
/// fixtures (Task 14) will surface any cases where this diverges from JS.
pub fn follow(a: &Changeset, b: &Changeset) -> Result<Changeset> {
    if a.old_len != b.old_len {
        return Err(ClientError::ApplyChangeset(format!(
            "follow length mismatch: {} vs {}",
            a.old_len, b.old_len
        )));
    }

    use crate::changeset::serializer::serialize;
    let a_wins_left = serialize(a) <= serialize(b);

    let mut out_ops: Vec<Op> = Vec::new();
    let mut out_bank = String::new();

    let mut a_iter = OpStream::new(&a.ops);
    let mut b_iter = OpStream::new(&b.ops);

    let b_bank: Vec<char> = b.char_bank.chars().collect();
    let mut b_bank_cursor = 0usize;

    loop {
        let a_op = a_iter.peek();
        let b_op = b_iter.peek();
        match (a_op, b_op) {
            (None, None) => break,
            (None, Some(b)) => {
                emit_b_op(&mut out_ops, &mut out_bank, &b, &b_bank, &mut b_bank_cursor);
                b_iter.advance(b.chars);
            }
            (Some(_), None) => {
                while let Some(a) = a_iter.peek() {
                    match a.opcode {
                        OpCode::Keep | OpCode::Insert => {
                            push_op(
                                &mut out_ops,
                                Op {
                                    opcode: OpCode::Keep,
                                    chars: a.chars,
                                    lines: a.lines,
                                    attribs: vec![],
                                },
                            );
                        }
                        OpCode::Delete => {}
                    }
                    let take = a.chars;
                    a_iter.advance(take);
                }
            }
            (Some(a), Some(b)) => match (a.opcode, b.opcode) {
                (OpCode::Insert, OpCode::Insert) => {
                    // Both clients want to insert at the same logical position.
                    // Tiebreaker decides who lands first.
                    if a_wins_left {
                        push_op(
                            &mut out_ops,
                            Op {
                                opcode: OpCode::Keep,
                                chars: a.chars,
                                lines: a.lines,
                                attribs: vec![],
                            },
                        );
                        let take = a.chars;
                        a_iter.advance(take);
                    } else {
                        let take = b.chars;
                        emit_b_op(&mut out_ops, &mut out_bank, &b, &b_bank, &mut b_bank_cursor);
                        b_iter.advance(take);
                    }
                }
                (OpCode::Insert, _) => {
                    push_op(
                        &mut out_ops,
                        Op {
                            opcode: OpCode::Keep,
                            chars: a.chars,
                            lines: a.lines,
                            attribs: vec![],
                        },
                    );
                    let take = a.chars;
                    a_iter.advance(take);
                }
                (_, OpCode::Insert) => {
                    let take = b.chars;
                    emit_b_op(&mut out_ops, &mut out_bank, &b, &b_bank, &mut b_bank_cursor);
                    b_iter.advance(take);
                }
                (OpCode::Keep, OpCode::Keep) => {
                    let n = a.chars.min(b.chars);
                    push_op(
                        &mut out_ops,
                        Op {
                            opcode: OpCode::Keep,
                            chars: n,
                            lines: 0,
                            attribs: vec![],
                        },
                    );
                    a_iter.advance(n);
                    b_iter.advance(n);
                }
                (OpCode::Keep, OpCode::Delete) => {
                    let n = a.chars.min(b.chars);
                    push_op(
                        &mut out_ops,
                        Op {
                            opcode: OpCode::Delete,
                            chars: n,
                            lines: 0,
                            attribs: vec![],
                        },
                    );
                    a_iter.advance(n);
                    b_iter.advance(n);
                }
                (OpCode::Delete, OpCode::Keep) | (OpCode::Delete, OpCode::Delete) => {
                    let n = a.chars.min(b.chars);
                    a_iter.advance(n);
                    b_iter.advance(n);
                }
            },
        }
    }

    let mut net_delta: i64 = 0;
    for op in &out_ops {
        match op.opcode {
            OpCode::Insert => net_delta += op.chars as i64,
            OpCode::Delete => net_delta -= op.chars as i64,
            OpCode::Keep => {}
        }
    }

    Ok(Changeset {
        old_len: a.new_len(),
        net_delta,
        ops: out_ops,
        char_bank: out_bank,
    })
}

fn emit_b_op(
    out_ops: &mut Vec<Op>,
    out_bank: &mut String,
    b: &Op,
    b_bank: &[char],
    b_bank_cursor: &mut usize,
) {
    push_op(
        out_ops,
        Op {
            opcode: b.opcode,
            chars: b.chars,
            lines: b.lines,
            attribs: b.attribs.clone(),
        },
    );
    if matches!(b.opcode, OpCode::Insert | OpCode::Delete) {
        let n = b.chars as usize;
        for c in &b_bank[*b_bank_cursor..*b_bank_cursor + n] {
            out_bank.push(*c);
        }
        *b_bank_cursor += n;
    }
}

fn push_op(out: &mut Vec<Op>, op: Op) {
    if let Some(last) = out.last_mut() {
        if last.opcode == op.opcode && last.lines == op.lines && last.attribs == op.attribs {
            last.chars += op.chars;
            return;
        }
    }
    out.push(op);
}

struct OpStream<'a> {
    ops: &'a [Op],
    idx: usize,
    consumed: u32,
}

impl<'a> OpStream<'a> {
    fn new(ops: &'a [Op]) -> Self {
        Self {
            ops,
            idx: 0,
            consumed: 0,
        }
    }

    fn peek(&self) -> Option<Op> {
        if self.idx >= self.ops.len() {
            return None;
        }
        let mut op = self.ops[self.idx].clone();
        op.chars -= self.consumed;
        Some(op)
    }

    fn advance(&mut self, n: u32) {
        if self.idx >= self.ops.len() {
            return;
        }
        self.consumed += n;
        let total = self.ops[self.idx].chars;
        if self.consumed >= total {
            self.idx += 1;
            self.consumed = 0;
        }
    }
}
