//! Regression: apply_remote MUST NOT crash with "follow length mismatch"
//! when an inbound remote arrives while we have stale entries in our
//! OutboundQueue (e.g., we typed N chars rapidly, some have been ACKed
//! server-side but the App-side queue hasn't drained them yet).
//!
//! Three scenarios:
//!   1. Empty queue, remote arrives — happy path (already covered elsewhere).
//!   2. Queue has consistent-chain pending [cs1(old=21 new=22), cs2(old=22
//!      new=23)] and a remote with old_len=21 arrives — must follow against
//!      cs1 first, then cs2, and apply.
//!   3. Queue has stale pending whose old_len doesn't match the remote's —
//!      simulating a missed drain. apply_remote MUST return an Err, not panic.
//!      (The App's drain_share_channels catches the Err and surfaces a flash
//!      message.)

use etherpad_client::changeset::parser::parse;
use etherpad_client::changeset::{Changeset, Op, OpCode};
use pad::buffer::{Buffer, CursorPos};
use pad::share::inbound::apply_remote;
use pad::share::outbound::OutboundQueue;
use tokio::sync::mpsc;

fn empty_queue() -> OutboundQueue {
    let (tx, _rx) = mpsc::unbounded_channel();
    OutboundQueue::new(tx)
}

fn queue_with(pending: Vec<Changeset>) -> OutboundQueue {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut q = OutboundQueue::new(tx);
    for cs in pending {
        q.send(cs).unwrap();
    }
    q
}

fn insert_at_end(old_text: &str, text: &str) -> Changeset {
    let old_len = old_text.chars().count() as u32;
    let inserted = text.chars().count() as u32;
    let mut ops = Vec::new();
    if old_len > 0 {
        let lines = old_text.chars().filter(|c| *c == '\n').count() as u32;
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: old_len,
            lines,
            attribs: vec![],
        });
    }
    ops.push(Op {
        opcode: OpCode::Insert,
        chars: inserted,
        lines: text.matches('\n').count() as u32,
        attribs: vec![],
    });
    Changeset {
        old_len,
        net_delta: inserted as i64,
        ops,
        char_bank: text.to_string(),
    }
}

#[test]
fn empty_queue_remote_applies_cleanly() {
    let mut buf = Buffer::from_text("hello");
    buf.move_cursor_to(CursorPos { line: 0, col: 5 });
    let remote = parse("Z:5>1=5+1$!").unwrap();
    apply_remote(&mut buf, &remote, &empty_queue()).unwrap();
    assert_eq!(buf.text(), "hello!");
}

#[test]
fn consistent_chain_pending_rebases_cleanly() {
    // Buffer is "abc" locally. We have two pending changesets:
    //   cs1: old="abc"(3), insert "1" at end -> "abc1"(4)
    //   cs2: old="abc1"(4), insert "2" at end -> "abc12"(5)
    // Buffer reflects post-cs2 = "abc12".
    // A remote NEW_CHANGES arrives with old_len=3 (server hadn't seen our cs's yet).
    // OT must rebase remote against cs1 then cs2; result applies on top.
    let mut buf = Buffer::from_text("abc12");
    buf.move_cursor_to(CursorPos { line: 0, col: 5 });
    let cs1 = insert_at_end("abc", "1");
    let cs2 = insert_at_end("abc1", "2");
    let queue = queue_with(vec![cs1, cs2]);
    let remote = insert_at_end("abc", "X"); // peer inserts X at end of original "abc"
    let r = apply_remote(&mut buf, &remote, &queue);
    assert!(
        r.is_ok(),
        "consistent pending chain should rebase cleanly, got {r:?}"
    );
    // Result should contain X somewhere appropriate. Exact position depends on
    // tiebreaker, but the text must contain all four characters.
    let final_text = buf.text();
    for c in ['a', 'b', 'c', '1', '2', 'X'] {
        assert!(final_text.contains(c), "missing {c} in {final_text:?}");
    }
}

#[test]
fn mismatched_pending_returns_err_not_panic() {
    // Simulates: we sent cs1 and got ACK, but didn't drain App-side queue.
    // Then a server-rebased NEW_CHANGES arrives with old_len = post-cs1 length,
    // but our queue still has cs1 with old_len = pre-cs1 length. Mismatch.
    let mut buf = Buffer::from_text("hello!");
    let stale = insert_at_end("hello", "!");
    let queue = queue_with(vec![stale]);
    // Remote thinks doc is "hello!" (post-our-cs1) — old_len=6, doesn't match
    // our cs1.old_len=5.
    let remote = insert_at_end("hello!", "?");
    let r = apply_remote(&mut buf, &remote, &queue);
    let err = r.expect_err("mismatched pending must surface as Err, got Ok");
    assert!(
        err.to_string().contains("length mismatch") || err.to_string().contains("follow"),
        "expected length-mismatch / follow error, got {err}"
    );
}
