use etherpad_client::changeset::parser::parse;
use etherpad_client::ot::{apply, inverse};
use proptest::prelude::*;

#[test]
fn inverse_of_insert_is_delete() {
    let a = parse("Z:0>3+3$XYZ").unwrap();
    let inv = inverse(&a, "").unwrap();
    let restored = apply(&inv, &apply(&a, "").unwrap()).unwrap();
    assert_eq!(restored, "");
}

#[test]
fn inverse_of_delete_is_insert() {
    let a = parse("Z:5<3=2-3$llo").unwrap();
    let inv = inverse(&a, "hello").unwrap();
    let restored = apply(&inv, &apply(&a, "hello").unwrap()).unwrap();
    assert_eq!(restored, "hello");
}

fn base36(mut n: u32) -> String {
    if n == 0 {
        return "0".into();
    }
    let mut out = Vec::new();
    while n > 0 {
        let d = (n % 36) as u8;
        out.push(if d < 10 {
            b'0' + d
        } else {
            b'a' + (d - 10)
        });
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

fn arb_text_and_delete_changeset() -> impl Strategy<Value = (String, String)> {
    "[a-z]{1,15}".prop_flat_map(|text| {
        let len = text.chars().count() as u32;
        (Just(text), 0u32..len).prop_map(move |(text, pos)| {
            let total_len = text.chars().count() as u32;
            let after = total_len - pos - 1;
            let deleted = text.chars().nth(pos as usize).unwrap();
            let mut wire = String::from("Z:");
            wire.push_str(&base36(total_len));
            wire.push('<');
            wire.push_str(&base36(1));
            if pos > 0 {
                wire.push('=');
                wire.push_str(&base36(pos));
            }
            wire.push('-');
            wire.push_str(&base36(1));
            if after > 0 {
                wire.push('=');
                wire.push_str(&base36(after));
            }
            wire.push('$');
            wire.push(deleted);
            (text, wire)
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..Default::default() })]
    #[test]
    fn inverse_round_trip((text, wire) in arb_text_and_delete_changeset()) {
        let cs = parse(&wire).unwrap();
        let inv = inverse(&cs, &text).unwrap();
        let applied = apply(&cs, &text).unwrap();
        let restored = apply(&inv, &applied).unwrap();
        prop_assert_eq!(restored, text);
    }
}
