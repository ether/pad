use super::{Changeset, Op, OpCode};
use crate::error::{ClientError, Result};

/// Parse the abbreviated Etherpad changeset format (v1 subset).
///
/// Grammar (BNF-ish):
/// ```text
/// changeset := "Z:" old_len delta_sign delta ops "$" bank
/// delta_sign := ">" | "<" | "="
/// ops := op*
/// op := attrib* lines? opcode count
/// attrib := "*" base36
/// lines := "|" base36
/// opcode := "+" | "-" | "="
/// count := base36
/// ```
///
/// NOT YET HANDLED: Etherpad's optional header line-bank (`|L<sign>R` immediately
/// after `delta`). Real-world Etherpad emits this when the document line count
/// changes — Task 14's conformance fixtures will surface any cases where this
/// matters, and the parser can be extended then. For the v1 subset, every `|N`
/// is treated as an op-prefix.
pub fn parse(input: &str) -> Result<Changeset> {
    let body = input
        .strip_prefix("Z:")
        .ok_or_else(|| ClientError::ParseChangeset("missing Z: prefix".into()))?;

    let (header, rest) = body
        .split_once('$')
        .ok_or_else(|| ClientError::ParseChangeset("missing $ char-bank separator".into()))?;

    let char_bank = rest.to_string();

    let mut chars = header.chars().peekable();
    let old_len = read_base36(&mut chars)?;

    let sign = chars
        .next()
        .ok_or_else(|| ClientError::ParseChangeset("missing delta sign".into()))?;
    let delta_abs = read_base36(&mut chars)? as i64;
    let net_delta = match sign {
        '>' => delta_abs,
        '<' => -delta_abs,
        '=' => 0,
        c => return Err(ClientError::ParseChangeset(format!("bad delta sign: {c}"))),
    };

    let mut ops = Vec::new();
    let mut pending_attribs: Vec<u32> = Vec::new();
    let mut pending_lines: u32 = 0;

    while let Some(&c) = chars.peek() {
        match c {
            '*' => {
                chars.next();
                pending_attribs.push(read_base36(&mut chars)?);
            }
            '|' => {
                chars.next();
                pending_lines = read_base36(&mut chars)?;
            }
            '+' | '-' | '=' => {
                chars.next();
                let opcode = match c {
                    '+' => OpCode::Insert,
                    '-' => OpCode::Delete,
                    '=' => OpCode::Keep,
                    _ => unreachable!(),
                };
                let count = read_base36(&mut chars)?;
                ops.push(Op {
                    opcode,
                    chars: count,
                    lines: pending_lines,
                    attribs: std::mem::take(&mut pending_attribs),
                });
                pending_lines = 0;
            }
            _ => {
                return Err(ClientError::ParseChangeset(format!(
                    "unexpected char in ops: {c}"
                )));
            }
        }
    }

    Ok(Changeset {
        old_len,
        net_delta,
        ops,
        char_bank,
    })
}

fn read_base36<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<u32> {
    let mut buf = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || (c.is_ascii_lowercase() && c <= 'z') {
            buf.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if buf.is_empty() {
        return Err(ClientError::ParseChangeset("expected base36 digit".into()));
    }
    u32::from_str_radix(&buf, 36)
        .map_err(|e| ClientError::ParseChangeset(format!("bad base36 '{buf}': {e}")))
}
