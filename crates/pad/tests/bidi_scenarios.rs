//! Comprehensive bidirectional sync test suite for pad ↔ Etherpad.
//!
//! Each test exercises one specific scenario the user might trigger.
//! Tests are bidirectional: terminal-typed content must reach the server,
//! and a parallel library-driven "browser" session must see it. A separate
//! `verify` session reads the final pad state.
//!
//! Skips automatically when PAD_ETHERPAD_BASE is unset.
//!
//! Run all: PAD_ETHERPAD_BASE=https://pad-dev.etherpad.org cargo test \
//!          -p pad --test bidi_scenarios -- --nocapture --test-threads=1
//!
//! Tests deliberately use a unique pad_id per case so failures don't
//! cross-contaminate. --test-threads=1 keeps the rate-limit footprint low.

use etherpad_client::Socket;
use etherpad_client::changeset::{Changeset, Op, OpCode};
use etherpad_client::session::{InboundEvent, PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use expectrl::{Eof, Expect, spawn};
use std::time::{Duration, Instant};

fn skip_if_no_remote() -> Option<String> {
    std::env::var("PAD_ETHERPAD_BASE").ok()
}

fn fresh_pad_id(kind: &str) -> String {
    format!(
        "pad-rust-{kind}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

async fn fresh_session(base: &str, pad_id: &str, token: &str) -> PadSession {
    let cookie = TungsteniteSocket::fetch_pad_cookie(base, pad_id)
        .await
        .expect("cookie");
    let mut socket = TungsteniteSocket::new(base, Some(cookie));
    socket.connect().await.expect("connect");
    let mut sess = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: pad_id.into(),
            token: token.into(),
            protocol_version: 2,
        },
    );
    sess.handshake().await.expect("handshake");
    sess
}

/// Read the pad's current text by opening a brand-new session and grabbing
/// initial_text. Detaches the session afterward.
async fn pad_text(base: &str, pad_id: &str) -> String {
    let mut sess = fresh_session(base, pad_id, "t.verify").await;
    let t = sess.initial_text().to_string();
    sess.disconnect().await.ok();
    t
}

/// Build an Etherpad-compatible "insert at end" changeset against `old_text`
/// for `insert`. Mirrors what pad::share::bridge does, but inline so tests
/// don't depend on the pad crate's internals.
fn cs_insert_at_end(old_text: &str, insert: &str) -> Changeset {
    let old_len = old_text.chars().count() as u32;
    let insert_chars = insert.chars().count() as u32;
    let kept_lines = old_text.chars().filter(|c| *c == '\n').count() as u32;
    let insert_lines = insert.matches('\n').count() as u32;
    let mut ops = Vec::new();
    if old_len > 0 {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: old_len,
            lines: kept_lines,
            attribs: vec![],
        });
    }
    ops.push(Op {
        opcode: OpCode::Insert,
        chars: insert_chars,
        lines: insert_lines,
        attribs: vec![],
    });
    Changeset {
        old_len,
        net_delta: insert_chars as i64,
        ops,
        char_bank: insert.to_string(),
    }
}

/// Spawn the pad binary against `url`. Returns the handle.
fn spawn_pad(url: &str) -> impl Expect {
    let bin = env!("CARGO_BIN_EXE_pad");
    let mut p = spawn(format!("{bin} {url}")).expect("spawn pad binary");
    p.set_expect_timeout(Some(Duration::from_secs(15)));
    // Allow handshake + raw-mode setup.
    std::thread::sleep(Duration::from_millis(2000));
    p
}

/// Exit the binary cleanly. Sends ^X then 'N' (don't save) to skip the
/// dirty-buffer prompt.
fn exit_pad(p: &mut impl Expect) {
    let _ = p.send([0x18u8].as_slice());
    std::thread::sleep(Duration::from_millis(200));
    let _ = p.send("N");
    let _ = p.expect(Eof);
}

/// Wait up to `timeout` for a fresh `verify` session to observe `marker`
/// in the pad text. Returns the final pad text seen (matched or not).
async fn poll_until_contains(base: &str, pad_id: &str, marker: &str, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    let mut last = String::new();
    while Instant::now() < deadline {
        last = pad_text(base, pad_id).await;
        if last.contains(marker) {
            return last;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    last
}

// ===========================================================================
// 1. SINGLE-CHAR TYPING — server receives typed chars in order
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn typing_single_chars_lands_in_order() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("single");
    let url = format!("{base}/p/{pad_id}");

    let mut p = spawn_pad(&url);
    let marker = "ABCDE";
    for c in marker.chars() {
        p.send([c as u8].as_slice()).expect("send char");
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(3000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains(marker),
        "expected {marker:?} in pad text, got: {final_text:?}"
    );
}

// ===========================================================================
// 2. ENTER + TYPE — typing on a new line works correctly
//    (user-reported: 'enter + hello world only showed h')
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn enter_then_type_on_new_line() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("enter");
    let url = format!("{base}/p/{pad_id}");

    let mut p = spawn_pad(&url);
    // Type chars on existing line first.
    for c in "first".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(80));
    }
    // Hit Enter, then type on new line.
    p.send([b'\r']).expect("send enter");
    std::thread::sleep(Duration::from_millis(150));
    for c in "second".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(80));
    }
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains("first"),
        "missing 'first' in {final_text:?}"
    );
    assert!(
        final_text.contains("second"),
        "missing 'second' in {final_text:?} — likely the compose-newline bug"
    );
}

// ===========================================================================
// 3. RAPID TYPING — many chars typed faster than ACK roundtrip,
//    BATCHED via compose, all reach the server
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rapid_typing_batches_correctly() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("rapid");
    let url = format!("{base}/p/{pad_id}");

    let mut p = spawn_pad(&url);
    let marker = "the-quick-brown-fox-jumps-over-the-lazy-dog";
    // No sleep between chars — exercise the batching path hard.
    for c in marker.chars() {
        p.send([c as u8].as_slice()).expect("send");
    }
    std::thread::sleep(Duration::from_millis(5000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains(marker),
        "rapid-typed {marker:?} missing — batching/compose corrupted it. Got: {final_text:?}"
    );
}

// ===========================================================================
// 4. BACKSPACE — terminal-deleted chars reach the server
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn backspace_propagates() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("bksp");
    let url = format!("{base}/p/{pad_id}");

    let mut p = spawn_pad(&url);
    for c in "AXXB".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(80));
    }
    // Backspace twice — should remove the two X's.
    p.send([0x7Fu8].as_slice()).expect("send bksp");
    std::thread::sleep(Duration::from_millis(150));
    p.send([0x7Fu8].as_slice()).expect("send bksp");
    std::thread::sleep(Duration::from_millis(150));
    p.send([0x7Fu8].as_slice()).expect("send bksp");
    std::thread::sleep(Duration::from_millis(150));
    // Now type A+something to mark.
    p.send([b'Z']).expect("send Z");
    std::thread::sleep(Duration::from_millis(3000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains("AZ"),
        "expected 'AZ' after AXXB + 3xBksp + Z, got: {final_text:?}"
    );
    assert!(
        !final_text.contains("AXXBZ"),
        "backspaces did not propagate: {final_text:?}"
    );
}

// ===========================================================================
// 5. BROWSER-SIM EDIT → terminal session must receive NEW_CHANGES
//    (already covered separately by integration_clear_propagates, but
//    re-asserted here so the suite covers both directions in one place)
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_sim_insert_propagates_to_terminal_session() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("recv");

    // Open session A (the "terminal" side, library-driven for assertion).
    let mut a = fresh_session(&base, &pad_id, "t.recv-A").await;
    let initial = a.initial_text().to_string();

    // Open session B and push an insert.
    let mut b = fresh_session(&base, &pad_id, "t.recv-B").await;
    let cs = cs_insert_at_end(&initial, "FROM-BROWSER\n");
    b.send_changeset(&cs).await.expect("B send");
    // Wait for B's ACK before disconnecting so the server has definitely
    // applied + queued broadcasts. Without this, the test races B's tear-
    // down against the server's NEW_CHANGES fan-out.
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(e) = b.pump_once_event().await {
            if matches!(e, InboundEvent::AckCommit { .. }) {
                return;
            }
        }
    })
    .await;
    b.disconnect().await.ok();

    // Drive A's inbound until we see the insert or timeout.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut got_it = false;
    while Instant::now() < deadline {
        let remaining = deadline.duration_since(Instant::now());
        match tokio::time::timeout(remaining, a.pump_once_event()).await {
            Ok(Ok(InboundEvent::Changeset(cs))) => {
                if cs.char_bank.contains("FROM-BROWSER") {
                    got_it = true;
                    break;
                }
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => panic!("A pump err: {e}"),
            Err(_) => break,
        }
    }
    a.disconnect().await.ok();
    assert!(got_it, "session A did not receive B's insert via NEW_CHANGES");
}

// ===========================================================================
// 6. CONCURRENT DELETE FROM BROWSER WHILE TERMINAL TYPES
//    Tests OT rebase in the inbound path.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_clear_while_terminal_idle() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("clear");

    // Pre-seed with content from session A.
    let mut a = fresh_session(&base, &pad_id, "t.clr-A").await;
    let initial = a.initial_text().to_string();
    let cs_seed = cs_insert_at_end(&initial, "SEED-TO-DELETE\n");
    a.send_changeset(&cs_seed).await.unwrap();
    // wait for ack
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(e) = a.pump_once_event().await {
            if matches!(e, InboundEvent::AckCommit { .. }) {
                return;
            }
        }
    })
    .await;
    a.disconnect().await.ok();

    // Confirm seeded.
    let after_seed = pad_text(&base, &pad_id).await;
    assert!(after_seed.contains("SEED-TO-DELETE"));

    // Session B clears everything except trailing \n.
    let mut b = fresh_session(&base, &pad_id, "t.clr-B").await;
    let pad_now = b.initial_text().to_string();
    let pad_len = pad_now.chars().count() as u32;
    let cs_clear = Changeset {
        old_len: pad_len,
        net_delta: -((pad_len - 1) as i64),
        ops: vec![Op {
            opcode: OpCode::Delete,
            chars: pad_len - 1,
            lines: pad_now.chars().take((pad_len - 1) as usize).filter(|c| *c == '\n').count() as u32,
            attribs: vec![],
        }],
        char_bank: String::new(),
    };
    b.send_changeset(&cs_clear).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(e) = b.pump_once_event().await {
            if matches!(e, InboundEvent::AckCommit { .. }) {
                return;
            }
        }
    })
    .await;
    b.disconnect().await.ok();

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        !final_text.contains("SEED-TO-DELETE"),
        "B's clear didn't remove SEED-TO-DELETE: {final_text:?}"
    );
}

// ===========================================================================
// 7. PASTE — large block of text arrives as one Event::Paste, one Changeset
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bracketed_paste_lands() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("paste");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    let body = "PASTE-https://pad-dev.etherpad.org/p/foo-bar-baz";
    let paste = format!("\x1b[200~{body}\x1b[201~");
    p.send(paste.as_str()).expect("send paste");
    std::thread::sleep(Duration::from_millis(3000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains(body),
        "paste body missing: {final_text:?}"
    );
}

// ===========================================================================
// 9. HUMAN-PACE TYPING — chars sent at ~100ms intervals (RTT-comparable) so
//    batches form and break naturally between strokes. Regression for the
//    user-reported scramble: typing 'interesting view change' arrived as
//    'e changew  viestingnteri' in the browser. If batching has a re-order
//    bug, this catches it.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn human_pace_typing_preserves_order() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("hpace");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    let marker = "interesting view change";
    for c in marker.chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(120));
    }
    std::thread::sleep(Duration::from_millis(5000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains(marker),
        "expected forward marker {marker:?} in pad text, got: {final_text:?}"
    );
}

// ===========================================================================
// 10. CTRL-K (Cut Line) — should both clear the local buffer line AND
//    propagate to the server (user-reported: clears in pad, not in browser)
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ctrl_k_cut_line_propagates() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("cutk");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    // Type a unique marker on its own line.
    for c in "CUT-LINE-MARKER".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(60));
    }
    // Hit Enter then ^K to cut the line we just typed... actually nano's
    // ^K cuts the CURRENT line. Since cursor's at end of typed text, ^K
    // cuts the line containing our marker. Wait for typed batch to land,
    // then ^K.
    std::thread::sleep(Duration::from_millis(2000));
    // Move cursor to start of the line so ^K cuts the marker not just
    // the trailing empty span. Home key (HOME = \x1b[H or ^A is more
    // common but pad uses arrow-left to move; simpler: just Ctrl-K from
    // wherever — nano cuts from cursor to end-of-line.
    p.send([0x0Bu8].as_slice()).expect("send ^K");
    std::thread::sleep(Duration::from_millis(3000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        !final_text.contains("CUT-LINE-MARKER"),
        "^K cut did not propagate to server — marker still present: {final_text:?}"
    );
}

// ===========================================================================
// 8. POLL HELPER — terminal change should be observable by simulator
//    within reasonable time (catches network task hangs)
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn terminal_change_observable_within_5s() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("poll");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    let marker = "POLL-MARKER";
    for c in marker.chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(40));
    }
    let final_text = poll_until_contains(&base, &pad_id, marker, Duration::from_secs(5)).await;
    exit_pad(&mut p);
    assert!(
        final_text.contains(marker),
        "{marker:?} not observed within 5s: {final_text:?}"
    );
}
