use etherpad_client::changeset::{parser::parse, serializer::serialize, Changeset, Op, OpCode};
use proptest::prelude::*;

#[test]
fn serialize_identity() {
    let cs = Changeset {
        old_len: 0,
        net_delta: 0,
        ops: vec![],
        char_bank: String::new(),
    };
    assert_eq!(serialize(&cs), "Z:0>0$");
}

#[test]
fn serialize_insert() {
    let cs = Changeset {
        old_len: 0,
        net_delta: 5,
        ops: vec![Op {
            opcode: OpCode::Insert,
            chars: 5,
            lines: 0,
            attribs: vec![],
        }],
        char_bank: "hello".into(),
    };
    assert_eq!(serialize(&cs), "Z:0>5+5$hello");
}

#[test]
fn serialize_keep_delete() {
    let cs = Changeset {
        old_len: 5,
        net_delta: -3,
        ops: vec![
            Op {
                opcode: OpCode::Keep,
                chars: 2,
                lines: 0,
                attribs: vec![],
            },
            Op {
                opcode: OpCode::Delete,
                chars: 3,
                lines: 0,
                attribs: vec![],
            },
        ],
        char_bank: "llo".into(),
    };
    assert_eq!(serialize(&cs), "Z:5<3=2-3$llo");
}

fn arb_op() -> impl Strategy<Value = Op> {
    (
        prop_oneof![Just(OpCode::Insert), Just(OpCode::Delete), Just(OpCode::Keep)],
        1u32..50,
        0u32..3,
        prop::collection::vec(0u32..16, 0..3),
    )
        .prop_map(|(opcode, chars, lines, attribs)| Op {
            opcode,
            chars,
            lines,
            attribs,
        })
}

fn arb_changeset() -> impl Strategy<Value = Changeset> {
    (
        0u32..200,
        prop::collection::vec(arb_op(), 0..8),
        "[a-zA-Z0-9]{0,40}",
    )
        .prop_map(|(old_len, ops, bank)| {
            let mut delta: i64 = 0;
            for op in &ops {
                match op.opcode {
                    OpCode::Insert => delta += op.chars as i64,
                    OpCode::Delete => delta -= op.chars as i64,
                    OpCode::Keep => {}
                }
            }
            Changeset {
                old_len,
                net_delta: delta,
                ops,
                char_bank: bank,
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 1024, ..Default::default() })]
    #[test]
    fn roundtrip(cs in arb_changeset()) {
        let wire = serialize(&cs);
        let back = parse(&wire).expect("parse roundtrip");
        prop_assert_eq!(back, cs);
    }
}
