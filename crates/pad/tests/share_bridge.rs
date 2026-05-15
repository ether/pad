use etherpad_client::changeset::OpCode;
use etherpad_client::ot::apply;
use pad::share::bridge::{changeset_for_delete, changeset_for_insert};

#[test]
fn insert_into_empty() {
    let cs = changeset_for_insert("", 0, "hi");
    assert_eq!(cs.old_len, 0);
    assert_eq!(cs.net_delta, 2);
    assert_eq!(cs.ops.len(), 1);
    assert_eq!(cs.ops[0].opcode, OpCode::Insert);
    assert_eq!(cs.char_bank, "hi");
    assert_eq!(apply(&cs, "").unwrap(), "hi");
}

#[test]
fn insert_in_middle() {
    let cs = changeset_for_insert("hello", 2, "X");
    assert_eq!(cs.old_len, 5);
    assert_eq!(cs.net_delta, 1);
    assert_eq!(apply(&cs, "hello").unwrap(), "heXllo");
}

#[test]
fn insert_at_end() {
    let cs = changeset_for_insert("hello", 5, "!");
    assert_eq!(apply(&cs, "hello").unwrap(), "hello!");
}

#[test]
fn delete_single() {
    let cs = changeset_for_delete("hello", 2, "l".into());
    assert_eq!(apply(&cs, "hello").unwrap(), "helo");
}

#[test]
fn delete_range() {
    let cs = changeset_for_delete("hello", 1, "ell".into());
    assert_eq!(apply(&cs, "hello").unwrap(), "ho");
}

#[test]
fn delete_at_start() {
    let cs = changeset_for_delete("hello", 0, "he".into());
    assert_eq!(apply(&cs, "hello").unwrap(), "llo");
}

#[test]
fn keep_with_newlines_emits_lines_prefix() {
    // Insert at the end of "a\nb\nc" — leading keep covers 5 chars with 2 newlines.
    let cs = changeset_for_insert("a\nb\nc", 5, "!");
    assert_eq!(cs.ops[0].opcode, OpCode::Keep);
    assert_eq!(cs.ops[0].lines, 2);
    assert_eq!(cs.ops[1].opcode, OpCode::Insert);
}
