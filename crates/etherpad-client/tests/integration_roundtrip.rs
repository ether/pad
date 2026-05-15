//! Round-trip end-to-end test against a real Etherpad: PadSession A pushes a
//! unique changeset, PadSession B opens the same pad fresh and asserts the
//! pushed text appears in its initial_text.
//!
//! Skips automatically if `PAD_ETHERPAD_BASE` is unset.

use etherpad_client::Socket;
use etherpad_client::session::{PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn push_then_read_back() {
    let Ok(base) = std::env::var("PAD_ETHERPAD_BASE") else {
        eprintln!("PAD_ETHERPAD_BASE unset, skipping");
        return;
    };
    let pad_id = format!(
        "pad-rust-roundtrip-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    eprintln!("target: {base}/p/{pad_id}");

    // Session A: push a unique marker.
    let cookie_a = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie A");
    let mut sock_a = TungsteniteSocket::new(&base, Some(cookie_a));
    sock_a.connect().await.expect("connect A");
    let mut sess_a = PadSession::new(
        Box::new(sock_a),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.roundtrip-A".into(),
            protocol_version: 2,
        },
    );
    sess_a.handshake().await.expect("handshake A");
    eprintln!(
        "session A: author={} rev={} initial={:?}",
        sess_a.author_id().as_str(),
        sess_a.rev(),
        sess_a.initial_text()
    );

    // Build an insert changeset against the current pad text.
    let initial_text = sess_a.initial_text().to_string();
    let initial_len = initial_text.chars().count() as u32;
    let marker = format!("RUST-ROUNDTRIP-MARKER-{}\n", uuid::Uuid::now_v7());
    let cs = pad_share_bridge_insert(&initial_text, initial_len, &marker);
    eprintln!(
        "outbound cs: old_len={} net_delta={} bank={:?}",
        cs.old_len, cs.net_delta, cs.char_bank
    );
    sess_a.send_changeset(&cs).await.expect("send_changeset A");

    // Pump session A for up to 5s to see what the server replies.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, sess_a.pump_once_event()).await {
            Ok(Ok(etherpad_client::InboundEvent::Changeset(cs))) => {
                eprintln!(
                    "A inbound changeset: old_len={} net_delta={} bank={:?}",
                    cs.old_len, cs.net_delta, cs.char_bank
                );
            }
            Ok(Ok(etherpad_client::InboundEvent::UserJoin { author_id, .. })) => {
                eprintln!("A presence join: {author_id}");
            }
            Ok(Ok(etherpad_client::InboundEvent::UserLeave { author_id })) => {
                eprintln!("A presence leave: {author_id}");
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                eprintln!("A pump error: {e}");
                break;
            }
            Err(_) => break,
        }
    }
    eprintln!("A final rev: {}", sess_a.rev());
    sess_a.disconnect().await.ok();

    // Session B: fresh connection, fresh cookie, asks the same pad.
    let cookie_b = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie B");
    let mut sock_b = TungsteniteSocket::new(&base, Some(cookie_b));
    sock_b.connect().await.expect("connect B");
    let mut sess_b = PadSession::new(
        Box::new(sock_b),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.roundtrip-B".into(),
            protocol_version: 2,
        },
    );
    sess_b.handshake().await.expect("handshake B");
    eprintln!(
        "session B: author={} rev={} initial.len={}",
        sess_b.author_id().as_str(),
        sess_b.rev(),
        sess_b.initial_text().chars().count()
    );

    assert!(
        sess_b.initial_text().contains(&marker.trim_end()),
        "expected marker {:?} in B's initial text, got {:?}",
        marker,
        sess_b.initial_text()
    );
    sess_b.disconnect().await.ok();
}

// Inline copy of the changeset_for_insert builder from pad's share::bridge —
// keeps etherpad-client free of a pad dep. Mirrors the same canonical form
// (trailing-keep implicit; leading keep includes |<newlines> when present).
fn pad_share_bridge_insert(
    old_text: &str,
    pos: u32,
    text: &str,
) -> etherpad_client::changeset::Changeset {
    use etherpad_client::changeset::{Changeset, Op, OpCode};
    let old_len = old_text.chars().count() as u32;
    let inserted = text.chars().count() as u32;
    let mut ops = Vec::new();
    if pos > 0 {
        let lines = old_text
            .chars()
            .take(pos as usize)
            .filter(|c| *c == '\n')
            .count() as u32;
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: pos,
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
