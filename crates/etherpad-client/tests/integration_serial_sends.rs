//! Verify that the serialize-on-ack pattern lets us push N changesets back-to-
//! back without the rev-race causing rejections. Drives a tokio task that
//! mimics what `pad::share::network::connect` does, then asserts session B
//! sees ALL N markers.

use etherpad_client::Socket;
use etherpad_client::changeset::{Changeset, Op, OpCode};
use etherpad_client::session::{InboundEvent, PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use std::collections::VecDeque;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ten_sequential_changesets_all_land() {
    let Ok(base) = std::env::var("PAD_ETHERPAD_BASE") else {
        eprintln!("PAD_ETHERPAD_BASE unset, skipping");
        return;
    };
    let pad_id = format!(
        "pad-rust-serial-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    eprintln!("target: {base}/p/{pad_id}");

    // Session A: queue 10 markers, drive them serially.
    let cookie_a = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie A");
    let mut sock_a = TungsteniteSocket::new(&base, Some(cookie_a));
    sock_a.connect().await.expect("connect A");
    let mut sess_a = PadSession::new(
        Box::new(sock_a),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.serial-A".into(),
            protocol_version: 2,
        },
    );
    sess_a.handshake().await.expect("handshake A");

    let mut text = sess_a.initial_text().to_string();

    // Build 10 markers and queue their changesets.
    let mut pending: VecDeque<Changeset> = VecDeque::new();
    for i in 0..10u32 {
        let marker = format!("ROUND-{i}\n");
        let pos = text.chars().count() as u32;
        let cs = build_insert(&text, pos, &marker);
        pending.push_back(cs);
        text.push_str(&marker);
    }

    // Send-and-wait loop: dispatch head, wait for ACCEPT_COMMIT, pop, repeat.
    let mut awaiting = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !pending.is_empty() || awaiting {
        if !awaiting && let Some(cs) = pending.pop_front() {
            sess_a.send_changeset(&cs).await.expect("send");
            awaiting = true;
            continue;
        }
        let remaining = deadline.duration_since(tokio::time::Instant::now());
        let Ok(evt) = tokio::time::timeout(remaining, sess_a.pump_once_event()).await else {
            panic!("timeout draining queue, {} still pending", pending.len());
        };
        match evt.expect("pump") {
            InboundEvent::AckCommit { new_rev } => {
                eprintln!("ack newRev={new_rev}");
                awaiting = false;
            }
            InboundEvent::Changeset(_) => {}
            _ => {}
        }
    }
    sess_a.disconnect().await.ok();

    // Session B: fresh — assert all 10 markers landed.
    let cookie_b = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie B");
    let mut sock_b = TungsteniteSocket::new(&base, Some(cookie_b));
    sock_b.connect().await.expect("connect B");
    let mut sess_b = PadSession::new(
        Box::new(sock_b),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.serial-B".into(),
            protocol_version: 2,
        },
    );
    sess_b.handshake().await.expect("handshake B");
    let final_text = sess_b.initial_text().to_string();
    eprintln!("B final text:\n{final_text}");
    for i in 0..10 {
        let needle = format!("ROUND-{i}");
        assert!(
            final_text.contains(&needle),
            "missing marker {needle}, got:\n{final_text}"
        );
    }
    sess_b.disconnect().await.ok();
}

fn build_insert(old_text: &str, pos: u32, text: &str) -> Changeset {
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
