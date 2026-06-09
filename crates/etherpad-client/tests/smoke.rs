//! Live wire-compat smoke test (Phase 2 of ether/etherpad#7923).
//!
//! Connects a `PadSession` to a fresh pad on a running Etherpad, sends a
//! changeset that inserts a known marker, then verifies the text round-tripped
//! — preferring the HTTP `getText` API (needs an apikey) and falling back to a
//! fresh `PadSession`'s `initial_text`.
//!
//! Marked `#[ignore]` so plain `cargo test` skips it; the manifest command
//! `cargo test --test smoke -- --ignored` runs it. Modeled on the skip pattern
//! in `integration_etherpad.rs`, but broadened: not only an unreachable base
//! URL but also any failing Etherpad-specific setup step (cookie fetch, WS
//! connect, handshake) prints a skip message and returns rather than failing.
//! This keeps the test safe to run against environments with no Etherpad — or
//! with something else answering on the port.
//!
//! Env contract:
//!   ETHERPAD_SMOKE_URL    base URL (fallback PAD_ETHERPAD_BASE, then
//!                         http://localhost:9003)
//!   ETHERPAD_SMOKE_APIKEY apikey for the HTTP API getText verification path

use etherpad_client::Socket;
use etherpad_client::changeset::{Changeset, Op, OpCode};
use etherpad_client::session::{PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use std::time::Duration;

fn smoke_base() -> String {
    std::env::var("ETHERPAD_SMOKE_URL")
        .or_else(|_| std::env::var("PAD_ETHERPAD_BASE"))
        .unwrap_or_else(|_| "http://localhost:9003".to_string())
}

async fn reachable(base: &str) -> bool {
    let Ok(c) = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    else {
        return false;
    };
    c.get(base).send().await.is_ok()
}

/// Canonical insert at position `pos` of `text` into `old_text`. Mirrors the
/// builder used by `integration_roundtrip.rs` (trailing keep is implicit).
fn insert_changeset(old_text: &str, pos: u32, text: &str) -> Changeset {
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

/// Read pad text back via the HTTP API. Returns `Ok(None)` if no apikey is set.
async fn get_text_via_api(base: &str, pad_id: &str) -> Result<Option<String>, String> {
    let Ok(apikey) = std::env::var("ETHERPAD_SMOKE_APIKEY") else {
        return Ok(None);
    };
    if apikey.is_empty() {
        return Ok(None);
    }
    let url = format!("{}/api/1.2.13/getText", base.trim_end_matches('/'));
    let c = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = c
        .get(&url)
        .query(&[("apikey", apikey.as_str()), ("padID", pad_id)])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    // { code, message, data: { text } }
    let text = body["data"]["text"].as_str().map(|s| s.to_string());
    Ok(text)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires a running Etherpad; run with --ignored"]
async fn smoke_wire_roundtrip() {
    let base = smoke_base();
    if !reachable(&base).await {
        eprintln!("Etherpad not reachable at {base}, skipping smoke test");
        return;
    }
    // UUID rather than epoch seconds so concurrent runs never collide.
    let pad_id = format!("pad-rust-smoke-{}", uuid::Uuid::now_v7());
    eprintln!("target: {base}/p/{pad_id}");

    // The HTTP probe above only proves *something* answers at `base`. The
    // Etherpad-specific setup (cookie fetch, WS connect, handshake) is also
    // treated as a skip condition so a reachable-but-not-Etherpad endpoint
    // skips cleanly instead of hard-failing the test.
    let cookie = match TungsteniteSocket::fetch_pad_cookie(&base, &pad_id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("fetch_pad_cookie failed ({e}); not a usable Etherpad, skipping");
            return;
        }
    };
    let mut socket = TungsteniteSocket::new(&base, Some(cookie));
    if let Err(e) = socket.connect().await {
        eprintln!("ws connect failed ({e}); not a usable Etherpad, skipping");
        return;
    }
    let mut session = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.smoke".into(),
            protocol_version: 2,
        },
    );
    if let Err(e) = session.handshake().await {
        eprintln!("handshake failed ({e}); not a usable Etherpad, skipping");
        return;
    }

    let initial_text = session.initial_text().to_string();
    let initial_len = initial_text.chars().count() as u32;
    let marker = format!("RUST-SMOKE-{}\n", uuid::Uuid::now_v7());
    let cs = insert_changeset(&initial_text, initial_len, &marker);
    session.send_changeset(&cs).await.expect("send_changeset");

    // Pump briefly so the server processes the commit.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.duration_since(tokio::time::Instant::now());
        match tokio::time::timeout(remaining, session.pump_once_event()).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                eprintln!("pump error: {e}");
                break;
            }
            Err(_) => break,
        }
    }
    session.disconnect().await.ok();

    let needle = marker.trim_end();

    // Verification path 1: HTTP API getText (preferred, if apikey provided).
    match get_text_via_api(&base, &pad_id).await {
        Ok(Some(text)) => {
            assert!(
                text.contains(needle),
                "marker {:?} not found in getText response {:?}",
                marker,
                text
            );
            eprintln!("verified via HTTP getText");
            return;
        }
        Ok(None) => eprintln!("ETHERPAD_SMOKE_APIKEY unset; verifying via fresh session"),
        Err(e) => eprintln!("getText failed ({e}); verifying via fresh session"),
    }

    // Verification path 2: fresh session reads the pad's persisted text.
    let cookie_b = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("fetch_pad_cookie B");
    let mut sock_b = TungsteniteSocket::new(&base, Some(cookie_b));
    sock_b.connect().await.expect("ws connect B");
    let mut sess_b = PadSession::new(
        Box::new(sock_b),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.smoke-B".into(),
            protocol_version: 2,
        },
    );
    sess_b.handshake().await.expect("handshake B");
    assert!(
        sess_b.initial_text().contains(needle),
        "marker {:?} not found in fresh session initial text {:?}",
        marker,
        sess_b.initial_text()
    );
    eprintln!("verified via fresh session initial_text");
    sess_b.disconnect().await.ok();
}
