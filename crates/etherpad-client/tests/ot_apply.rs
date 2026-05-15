use etherpad_client::changeset::parser::parse;
use etherpad_client::ot::apply;

#[test]
fn apply_identity_to_empty() {
    let cs = parse("Z:0>0$").unwrap();
    assert_eq!(apply(&cs, "").unwrap(), "");
}

#[test]
fn apply_insert_to_empty() {
    let cs = parse("Z:0>5+5$hello").unwrap();
    assert_eq!(apply(&cs, "").unwrap(), "hello");
}

#[test]
fn apply_keep_delete() {
    let cs = parse("Z:5<3=2-3$llo").unwrap();
    assert_eq!(apply(&cs, "hello").unwrap(), "he");
}

#[test]
fn apply_keep_insert_keep() {
    let cs = parse("Z:5>1=2+1=3$Y").unwrap();
    assert_eq!(apply(&cs, "hello").unwrap(), "heYllo");
}

#[test]
fn apply_old_len_mismatch_errs() {
    let cs = parse("Z:5>1=2+1=3$Y").unwrap();
    assert!(apply(&cs, "hi").is_err());
}

#[test]
fn apply_short_char_bank_errs() {
    let cs = parse("Z:0>5+5$ab").unwrap();
    assert!(apply(&cs, "").is_err());
}
