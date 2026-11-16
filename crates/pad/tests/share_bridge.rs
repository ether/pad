use etherpad_client::changeset::OpCode;
use etherpad_client::changeset::serializer::serialize;
use etherpad_client::ot::apply;
use pad::share::bridge::{changeset_for_delete, changeset_for_insert, changeset_for_replace};

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
    // Insert at the end of "a\nb\nc" — leading keep covers 5 chars with 2
    // newlines but ends mid-line, so it splits into `|2=4` + `=1`.
    let cs = changeset_for_insert("a\nb\nc", 5, "!");
    assert_eq!(cs.ops[0].opcode, OpCode::Keep);
    assert_eq!(cs.ops[0].lines, 2);
    assert_eq!(cs.ops[1].opcode, OpCode::Keep);
    assert_eq!(cs.ops[1].lines, 0);
    assert_eq!(cs.ops[2].opcode, OpCode::Insert);
}

// ether/pad#3: multi-line `=` / `-` ops must end on `\n`; a run that ends
// mid-line is split at its last newline.

#[test]
fn keep_ending_mid_line_is_split() {
    // The issue's example: 65 (`1t`) kept chars over 5 newlines, ending one
    // char into the next line. `|5=1t` must become `|5=1s=1`.
    let lines = "abcdefghijkl\n".repeat(4) + "abcdefghijk\n"; // 64 chars
    let old = format!("{lines}xy\n");
    let cs = changeset_for_insert(&old, 65, "Z");
    assert_eq!(serialize(&cs), "Z:1v>1|5=1s=1+1$Z");
    assert_eq!(apply(&cs, &old).unwrap(), format!("{lines}xZy\n"));
}

#[test]
fn delete_ending_mid_line_is_split() {
    let old = "ab\ncd\nef";
    let cs = changeset_for_delete(old, 0, "ab\ncd\ne".into());
    assert_eq!(serialize(&cs), "Z:8<7|2-6-1$");
    assert_eq!(apply(&cs, old).unwrap(), "f");
}

#[test]
fn keep_and_delete_both_split() {
    let old = "ab\ncd\nef\ngh";
    // Keep "ab\nc" (|1=3 =1), delete "d\nef\ng" (|2-5 -1).
    let cs = changeset_for_delete(old, 4, "d\nef\ng".into());
    assert_eq!(serialize(&cs), "Z:b<6|1=3=1|2-5-1$");
    assert_eq!(apply(&cs, old).unwrap(), "ab\nch");
}

#[test]
fn replace_splits_keep_delete_and_insert() {
    let old = "ab\ncd\nef";
    let cs = changeset_for_replace(old, 1, "b\nc", "X\nY");
    assert_eq!(serialize(&cs), "Z:8>0=1|1-2-1|1+2+1$X\nY");
    assert_eq!(apply(&cs, old).unwrap(), "aX\nYd\nef");
}

#[test]
fn canonical_runs_are_not_split() {
    // Keep and delete both already end on `\n`: single ops, unchanged.
    let old = "ab\ncd\nef\n";
    let cs = changeset_for_delete(old, 3, "cd\n".into());
    assert_eq!(serialize(&cs), "Z:9<3|1=3|1-3$");
    assert_eq!(apply(&cs, old).unwrap(), "ab\nef\n");

    // Single-line runs carry no `|` prefix and stay single ops.
    let cs = changeset_for_insert("hello", 2, "X");
    assert_eq!(serialize(&cs), "Z:5>1=2+1$X");
}
