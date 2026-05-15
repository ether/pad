use super::{Changeset, Op, OpCode};

pub fn serialize(cs: &Changeset) -> String {
    let mut out = String::with_capacity(32 + cs.char_bank.len());
    out.push_str("Z:");
    out.push_str(&to_base36(cs.old_len));

    if cs.net_delta > 0 {
        out.push('>');
        out.push_str(&to_base36(cs.net_delta as u32));
    } else if cs.net_delta < 0 {
        out.push('<');
        out.push_str(&to_base36((-cs.net_delta) as u32));
    } else {
        out.push('>');
        out.push_str(&to_base36(0));
    }

    // v1 does not emit a header line-bank — line counts live per-op only.
    // Symmetric with the parser; conformance fixtures (Task 14) will catch
    // any cases where JS-emitted changesets require us to support it.

    for op in &cs.ops {
        emit_op(&mut out, op);
    }

    out.push('$');
    out.push_str(&cs.char_bank);
    out
}

fn emit_op(out: &mut String, op: &Op) {
    for a in &op.attribs {
        out.push('*');
        out.push_str(&to_base36(*a));
    }
    if op.lines > 0 {
        out.push('|');
        out.push_str(&to_base36(op.lines));
    }
    out.push(match op.opcode {
        OpCode::Insert => '+',
        OpCode::Delete => '-',
        OpCode::Keep => '=',
    });
    out.push_str(&to_base36(op.chars));
}

fn to_base36(n: u32) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut buf = Vec::new();
    let mut n = n;
    while n > 0 {
        let d = (n % 36) as u8;
        buf.push(if d < 10 {
            b'0' + d
        } else {
            b'a' + (d - 10)
        });
        n /= 36;
    }
    buf.reverse();
    String::from_utf8(buf).expect("base36 ASCII")
}
