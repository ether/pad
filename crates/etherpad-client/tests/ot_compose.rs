use etherpad_client::changeset::parser::parse;
use etherpad_client::ot::{apply, compose};

#[test]
fn compose_two_inserts() {
    let a = parse("Z:0>2+2$AB").unwrap();
    let b = parse("Z:2>1=2+1$C").unwrap();
    let c = compose(&a, &b).unwrap();
    assert_eq!(apply(&c, "").unwrap(), "ABC");
    assert_eq!(
        apply(&c, "").unwrap(),
        apply(&b, &apply(&a, "").unwrap()).unwrap()
    );
}

#[test]
fn compose_insert_then_delete() {
    let a = parse("Z:0>3+3$XYZ").unwrap();
    let b = parse("Z:3<1=1-1=1$Y").unwrap();
    let c = compose(&a, &b).unwrap();
    assert_eq!(apply(&c, "").unwrap(), "XZ");
}
