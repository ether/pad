//! Comprehensive end-to-end against pad-dev: drive the actual `pad` binary
//! in a PTY, simultaneously drive a library "browser" session, and verify
//! BOTH directions of sync work on a FRESH pad (no prior corruption).
//!
//! Three things this checks:
//!   1. Terminal-typed chars reach the simulated browser session.
//!   2. Simulated-browser edits reach the terminal pad (binary doesn't
//!      print them itself, but we verify by quitting and reading the
//!      saved file OR by quitting the binary first and inspecting the
//!      server state for the browser's edits).
//!   3. Times the first-roundtrip so we can quantify "slow".

use etherpad_client::Socket;
use etherpad_client::changeset::{Changeset, Op, OpCode};
use etherpad_client::session::{InboundEvent, PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use expectrl::{Eof, Expect, spawn};
use std::time::{Duration, Instant};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fresh_pad_bidir_sync() {
    let Ok(base) = std::env::var("PAD_ETHERPAD_BASE") else {
        eprintln!("PAD_ETHERPAD_BASE unset, skipping");
        return;
    };
    let pad_id = format!(
        "pad-rust-bidir-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let url = format!("{base}/p/{pad_id}");
    eprintln!("FRESH pad: {url}");

    // Spawn the binary as the "terminal" session.
    let bin = env!("CARGO_BIN_EXE_pad");
    let mut p = spawn(format!("{bin} {url}")).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(15)));
    std::thread::sleep(Duration::from_millis(2000));

    // Open a library session as the "browser" — same pad.
    let cookie = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie");
    let mut sock = TungsteniteSocket::new(&base, Some(cookie));
    sock.connect().await.expect("connect");
    let mut browser = PadSession::new(
        Box::new(sock),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.browser-sim".into(),
            protocol_version: 2,
        },
    );
    browser.handshake().await.expect("browser handshake");
    let baseline_text = browser.initial_text().to_string();
    let baseline_len = baseline_text.chars().count() as u32;
    eprintln!("baseline ({baseline_len} chars): {baseline_text:?}");

    // === Phase 1: terminal types a marker; measure round-trip ===
    let marker_term = format!(
        "T{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() % 1000
    );
    let typing_start = Instant::now();
    for c in marker_term.chars() {
        p.send([c as u8].as_slice()).expect("send char");
    }
    // Pump browser until we see the marker (or timeout 10s).
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut term_chars_seen = 0usize;
    let mut last_text = String::new();
    while Instant::now() < deadline && term_chars_seen < marker_term.len() {
        let remaining = deadline.duration_since(Instant::now());
        match tokio::time::timeout(remaining, browser.pump_once_event()).await {
            Ok(Ok(InboundEvent::Changeset(_))) => {
                // After any inbound, the rev advanced. Re-handshake B fresh is
                // heavy — just track inferred state from the cs. Simpler: peek
                // at internal state every iteration via initial_text() only at
                // start; for this test, count inbound changesets.
                term_chars_seen += 1;
                last_text.push('?'); // placeholder
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => panic!("browser pump err: {e}"),
            Err(_) => break,
        }
    }
    let term_to_browser_ms = typing_start.elapsed().as_millis();
    eprintln!(
        "terminal → browser: {} chars in {} ms ({} cs received)",
        marker_term.len(),
        term_to_browser_ms,
        term_chars_seen
    );

    // Open a fresh verification session to read the actual pad state.
    let verify_cookie = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .unwrap();
    let mut verify_sock = TungsteniteSocket::new(&base, Some(verify_cookie));
    verify_sock.connect().await.unwrap();
    let mut verify = PadSession::new(
        Box::new(verify_sock),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.verify".into(),
            protocol_version: 2,
        },
    );
    verify.handshake().await.unwrap();
    let phase1_text = verify.initial_text().to_string();
    verify.disconnect().await.ok();
    eprintln!("after terminal type, pad text: {phase1_text:?}");
    assert!(
        phase1_text.contains(&marker_term),
        "terminal-typed marker {marker_term:?} not in pad text {phase1_text:?}"
    );

    // === Phase 2: browser sends an edit; verify terminal receives it ===
    // We can't easily inspect the binary's buffer mid-flight, so instead
    // we verify the browser's edit reaches the SERVER (which means the
    // terminal would receive it via NEW_CHANGES too — that path is already
    // covered by integration_clear_propagates.rs).
    // Marker ends with \n to dodge Etherpad's auto-trailing-newline
    // insertion (which splits non-\n-terminated text at length-1).
    let marker_browser = format!(
        "B{}\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() % 1000
    );
    let pad_now_len = phase1_text.chars().count() as u32;
    let kept_lines = phase1_text.chars().filter(|c| *c == '\n').count() as u32;
    let marker_lines = marker_browser.matches('\n').count() as u32;
    let cs_browser = Changeset {
        old_len: pad_now_len,
        net_delta: marker_browser.len() as i64,
        ops: vec![
            Op {
                opcode: OpCode::Keep,
                chars: pad_now_len,
                lines: kept_lines,
                attribs: vec![],
            },
            Op {
                opcode: OpCode::Insert,
                chars: marker_browser.chars().count() as u32,
                lines: marker_lines,
                attribs: vec![],
            },
        ],
        char_bank: marker_browser.clone(),
    };
    browser.send_changeset(&cs_browser).await.expect("browser send");
    let browser_ack_start = Instant::now();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut browser_acked = false;
    while Instant::now() < deadline {
        let remaining = deadline.duration_since(Instant::now());
        match tokio::time::timeout(remaining, browser.pump_once_event()).await {
            Ok(Ok(InboundEvent::AckCommit { new_rev })) => {
                eprintln!("browser ACK newRev={new_rev}");
                browser_acked = true;
                break;
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => panic!("browser pump err: {e}"),
            Err(_) => break,
        }
    }
    let browser_rt_ms = browser_ack_start.elapsed().as_millis();
    eprintln!("browser send → ack: {browser_rt_ms} ms");
    assert!(browser_acked, "browser edit was NOT acked — likely a corruption");

    // Exit binary cleanly.
    p.send([0x18u8].as_slice()).expect("send ^X");
    std::thread::sleep(Duration::from_millis(200));
    p.send("N").expect("send N");
    let _ = p.expect(Eof);
    browser.disconnect().await.ok();

    // Final state check.
    let final_cookie = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .unwrap();
    let mut final_sock = TungsteniteSocket::new(&base, Some(final_cookie));
    final_sock.connect().await.unwrap();
    let mut final_sess = PadSession::new(
        Box::new(final_sock),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.final".into(),
            protocol_version: 2,
        },
    );
    final_sess.handshake().await.unwrap();
    let final_text = final_sess.initial_text().to_string();
    final_sess.disconnect().await.ok();
    eprintln!("final pad text: {final_text:?}");
    assert!(
        final_text.contains(&marker_term),
        "terminal marker missing"
    );
    assert!(
        final_text.contains(marker_browser.trim_end()),
        "browser marker {:?} missing from final pad text:\n{final_text}",
        marker_browser.trim_end()
    );
}
