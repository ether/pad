use etherpad_client::changeset::Changeset;
use etherpad_client::changeset::parser::parse;
use etherpad_client::ot::{apply, follow};
use proptest::prelude::*;

#[test]
fn follow_concurrent_inserts_diverge_to_same_text() {
    // Both clients start with "hello".
    // Client A inserts "A" at position 0 → "Ahello".
    // Client B inserts "B" at position 5 → "helloB".
    let a = parse("Z:5>1+1=5$A").unwrap();
    let b = parse("Z:5>1=5+1$B").unwrap();

    let b_prime = follow(&a, &b).unwrap();
    let a_prime = follow(&b, &a).unwrap();

    let after_a_then_b = apply(&b_prime, &apply(&a, "hello").unwrap()).unwrap();
    let after_b_then_a = apply(&a_prime, &apply(&b, "hello").unwrap()).unwrap();

    assert_eq!(after_a_then_b, after_b_then_a);
}

fn base36(mut n: u32) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut out = Vec::new();
    while n > 0 {
        let d = (n % 36) as u8;
        out.push(if d < 10 { b'0' + d } else { b'a' + (d - 10) });
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

fn make_insert_at(text: &str, pos: usize, ins: &str) -> Changeset {
    let len = text.chars().count() as u32;
    let pos = pos as u32;
    let ins_len = ins.chars().count() as u32;
    let after = len - pos;
    let mut wire = String::from("Z:");
    wire.push_str(&base36(len));
    wire.push('>');
    wire.push_str(&base36(ins_len));
    if pos > 0 {
        wire.push('=');
        wire.push_str(&base36(pos));
    }
    wire.push('+');
    wire.push_str(&base36(ins_len));
    if after > 0 {
        wire.push('=');
        wire.push_str(&base36(after));
    }
    wire.push('$');
    wire.push_str(ins);
    parse(&wire).unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..Default::default() })]
    #[test]
    fn convergence(
        text in "[a-z]{5,15}",
        pos_a in 0usize..15,
        pos_b in 0usize..15,
        ins_a in "[A-M]{1,3}",
        ins_b in "[N-Z]{1,3}",
    ) {
        let len = text.chars().count();
        let pa = pos_a % (len + 1);
        let pb = pos_b % (len + 1);
        let a = make_insert_at(&text, pa, &ins_a);
        let b = make_insert_at(&text, pb, &ins_b);
        let b_prime = follow(&a, &b).unwrap();
        let a_prime = follow(&b, &a).unwrap();
        let aab = apply(&b_prime, &apply(&a, &text).unwrap()).unwrap();
        let aba = apply(&a_prime, &apply(&b, &text).unwrap()).unwrap();
        prop_assert_eq!(aab, aba);
    }
}
