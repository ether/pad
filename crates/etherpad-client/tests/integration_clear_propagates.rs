//! Regression: when a second client (e.g. the browser) clears a shared pad,
//! the first client must receive the NEW_CHANGES and update its local view.
//!
//! User-reported: clearing in browser → terminal pad still shows old content.

use etherpad_client::Socket;
use etherpad_client::changeset::{Changeset, Op, OpCode};
use etherpad_client::session::{InboundEvent, PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn second_client_clear_propagates_to_first() {
    let Ok(base) = std::env::var("PAD_ETHERPAD_BASE") else {
        eprintln!("PAD_ETHERPAD_BASE unset, skipping");
        return;
    };
    let pad_id = format!(
        "pad-rust-clear-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    eprintln!("target: {base}/p/{pad_id}");

    // --- Client A (the "terminal") --------------------------------------
    let cookie_a = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie A");
    let mut sock_a = TungsteniteSocket::new(&base, Some(cookie_a));
    sock_a.connect().await.expect("connect A");
    let mut sess_a = PadSession::new(
        Box::new(sock_a),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.clear-A".into(),
            protocol_version: 2,
        },
    );
    sess_a.handshake().await.expect("handshake A");
    let initial_a = sess_a.initial_text().to_string();
    let initial_len = initial_a.chars().count() as u32;
    eprintln!("A initial ({initial_len} chars)");

    // --- Client B (the "browser") sends a clear-everything changeset ----
    // The wire form Etherpad's web client uses for "Clear authorship colors
    // and content" is roughly: delete all chars (keeping the final \n), so
    // the doc becomes a single newline.
    let cookie_b = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie B");
    let mut sock_b = TungsteniteSocket::new(&base, Some(cookie_b));
    sock_b.connect().await.expect("connect B");
    let mut sess_b = PadSession::new(
        Box::new(sock_b),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.clear-B".into(),
            protocol_version: 2,
        },
    );
    sess_b.handshake().await.expect("handshake B");
    let initial_b = sess_b.initial_text().to_string();
    assert_eq!(initial_a, initial_b, "both clients see same starting state");

    // Build a "delete everything except trailing newline" changeset against
    // B's view. We delete (initial_len - 1) chars from offset 0, leaving the
    // trailing \n. Etherpad auto-appends the trailing newline so this should
    // produce a pad text of "\n".
    if initial_len < 1 {
        eprintln!("pad came up empty, skipping (server welcome text changed?)");
        return;
    }
    let delete_count = initial_len - 1;
    let delete_text: String = initial_a.chars().take(delete_count as usize).collect();
    let delete_lines = delete_text.matches('\n').count() as u32;
    let cs_clear = Changeset {
        old_len: initial_len,
        net_delta: -(delete_count as i64),
        ops: vec![Op {
            opcode: OpCode::Delete,
            chars: delete_count,
            lines: delete_lines,
            attribs: vec![],
        }],
        // Etherpad canonical form: Delete consumes from source text, NOT bank.
        char_bank: String::new(),
    };
    sess_b.send_changeset(&cs_clear).await.expect("B send clear");
    eprintln!("B sent clear");

    // Pump B for a moment to see if server ACKed or disconnected us. This
    // is the diagnostic missing from the earlier failure — we were closing B
    // immediately after send and not noticing the badChangeset disconnect.
    let b_deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    let mut b_acked = false;
    while tokio::time::Instant::now() < b_deadline {
        let remaining = b_deadline.duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, sess_b.pump_once_event()).await {
            Ok(Ok(InboundEvent::AckCommit { new_rev })) => {
                eprintln!("B got ACK_COMMIT newRev={new_rev}");
                b_acked = true;
                break;
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                eprintln!("B pump error (likely disconnect): {e}");
                break;
            }
            Err(_) => break,
        }
    }
    if !b_acked {
        eprintln!("B's delete was NOT ACKed by server — likely rejected as badChangeset");
    }
    sess_b.disconnect().await.ok();

    // --- Pump A for up to 10s; assert we receive the clear NEW_CHANGES --
    let mut got_clear = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, sess_a.pump_once_event()).await {
            Ok(Ok(InboundEvent::Changeset(cs))) => {
                eprintln!(
                    "A inbound cs: old_len={} net_delta={} ops={} bank.len={}",
                    cs.old_len,
                    cs.net_delta,
                    cs.ops.len(),
                    cs.char_bank.len()
                );
                if cs.net_delta < 0 && cs.ops.iter().any(|o| matches!(o.opcode, OpCode::Delete))
                {
                    got_clear = true;
                    break;
                }
            }
            Ok(Ok(other)) => {
                let _ = other;
            }
            Ok(Err(e)) => {
                panic!("A pump error: {e}");
            }
            Err(_) => break,
        }
    }
    sess_a.disconnect().await.ok();

    assert!(
        got_clear,
        "client A did not receive the clear NEW_CHANGES from server"
    );
}
