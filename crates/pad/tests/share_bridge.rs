use etherpad_client::changeset::OpCode;
use etherpad_client::ot::apply;
use pad::share::bridge::{changeset_for_delete, changeset_for_insert};

#[test]
fn insert_into_empty() {
    let cs = changeset_for_insert(0, 0, "hi");
    assert_eq!(cs.old_len, 0);
    assert_eq!(cs.net_delta, 2);
    assert_eq!(cs.ops.len(), 1);
    assert_eq!(cs.ops[0].opcode, OpCode::Insert);
    assert_eq!(cs.char_bank, "hi");
    assert_eq!(apply(&cs, "").unwrap(), "hi");
}

#[test]
fn insert_in_middle() {
    let cs = changeset_for_insert(5, 2, "X");
    assert_eq!(cs.old_len, 5);
    assert_eq!(cs.net_delta, 1);
    assert_eq!(apply(&cs, "hello").unwrap(), "heXllo");
}

#[test]
fn insert_at_end() {
    let cs = changeset_for_insert(5, 5, "!");
    assert_eq!(apply(&cs, "hello").unwrap(), "hello!");
}

#[test]
fn delete_single() {
    let cs = changeset_for_delete(5, 2, "l".into());
    assert_eq!(apply(&cs, "hello").unwrap(), "helo");
}

#[test]
fn delete_range() {
    let cs = changeset_for_delete(5, 1, "ell".into());
    assert_eq!(apply(&cs, "hello").unwrap(), "ho");
}

#[test]
fn delete_at_start() {
    let cs = changeset_for_delete(5, 0, "he".into());
    assert_eq!(apply(&cs, "hello").unwrap(), "llo");
}
