use etherpad_client::changeset::{parser::parse, OpCode};

#[test]
fn parse_empty_identity() {
    // No-op changeset over an empty doc.
    let cs = parse("Z:0>0$").expect("parse identity");
    assert_eq!(cs.old_len, 0);
    assert_eq!(cs.net_delta, 0);
    assert!(cs.ops.is_empty());
    assert_eq!(cs.char_bank, "");
}

#[test]
fn parse_simple_insert() {
    // Insert "hello" into empty doc.
    let cs = parse("Z:0>5+5$hello").expect("parse insert");
    assert_eq!(cs.old_len, 0);
    assert_eq!(cs.net_delta, 5);
    assert_eq!(cs.ops.len(), 1);
    assert_eq!(cs.ops[0].opcode, OpCode::Insert);
    assert_eq!(cs.ops[0].chars, 5);
    assert_eq!(cs.char_bank, "hello");
}

#[test]
fn parse_keep_then_delete() {
    // In "hello": keep 2 ("he"), delete 3 ("llo").
    let cs = parse("Z:5<3=2-3$llo").expect("parse keep+delete");
    assert_eq!(cs.old_len, 5);
    assert_eq!(cs.net_delta, -3);
    assert_eq!(cs.ops.len(), 2);
    assert_eq!(cs.ops[0].opcode, OpCode::Keep);
    assert_eq!(cs.ops[0].chars, 2);
    assert_eq!(cs.ops[1].opcode, OpCode::Delete);
    assert_eq!(cs.ops[1].chars, 3);
    assert_eq!(cs.char_bank, "llo");
}

#[test]
fn parse_op_prefix_newlines() {
    // Keep 1 char, insert "X\n" (op-prefix |1 says insert spans 1 newline), keep 2.
    let cs = parse("Z:3>2=1|1+2=2$X\n").expect("parse op-prefix newlines");
    assert_eq!(cs.old_len, 3);
    assert_eq!(cs.net_delta, 2);
    assert_eq!(cs.ops.len(), 3);
    assert_eq!(cs.ops[0].lines, 0);
    assert_eq!(cs.ops[1].lines, 1);
    assert_eq!(cs.ops[2].lines, 0);
}

#[test]
fn parse_with_attribs() {
    // *0 marks "bold" on the inserted "X".
    let cs = parse("Z:0>1*0+1$X").expect("parse attribs");
    assert_eq!(cs.ops[0].attribs, vec![0]);
}

#[test]
fn parse_malformed_no_z_prefix() {
    let err = parse(":0>0$").unwrap_err();
    assert!(err.to_string().contains("Z"));
}

#[test]
fn parse_malformed_missing_dollar() {
    let err = parse("Z:0>0").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains('$') || msg.to_lowercase().contains("bank"));
}
