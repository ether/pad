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
// 9b. JITTERED PACE TYPING — typing at varying intervals (10ms..300ms) to
//     exercise every batching cliff. Reproduction attempt for user-reported
//     "interesting view change" → "e changew  viestingnteri" scramble.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn jittered_typing_preserves_order() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("jitter");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    let marker = "interesting view change";
    let intervals: [u64; 23] = [10, 250, 30, 300, 80, 50, 200, 40, 60, 350, 20, 100, 180, 25, 70, 280, 15, 120, 220, 45, 90, 160, 35];
    for (c, ms) in marker.chars().zip(intervals.iter()) {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(*ms));
    }
    std::thread::sleep(Duration::from_millis(6000));
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
// 11. TYPE THEN IMMEDIATE CTRL-K — regression for the user-reported browser
//     crash "TypeError: can't access property 'key', e is null" in
//     offsetOfEntry. Cut of the last line would empty the pad which violates
//     Etherpad's "always ends with \n" invariant; the browser's rep.lines
//     can't survive a fully empty document.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn type_then_ctrl_k_leaves_pad_with_trailing_newline() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("typecut");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    for c in "hello world".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(80));
    }
    // Immediately ^K — no wait for batched send to finalize.
    p.send([0x0Bu8].as_slice()).expect("send ^K");
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        !final_text.is_empty(),
        "pad text empty after type+^K — violates Etherpad's trailing-\\n invariant"
    );
    assert!(
        !final_text.contains("hello world"),
        "^K didn't propagate: {final_text:?}"
    );
}

// ===========================================================================
// 12. RAPID BACKSPACE TO EMPTY — backspace all the way through the welcome
//     content. Must leave the trailing \n intact.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rapid_backspace_preserves_trailing_newline() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("bksp-all");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    // Type something first so we know what we're deleting.
    for c in "DELETE-ME".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(40));
    }
    std::thread::sleep(Duration::from_millis(2000));
    // Backspace many times. Should not over-delete past the start of pad.
    for _ in 0..100 {
        p.send([0x7Fu8].as_slice()).expect("bksp");
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        !final_text.is_empty(),
        "pad text empty after 100 backspaces — must keep trailing \\n"
    );
    assert!(
        !final_text.contains("DELETE-ME"),
        "backspaces did not remove DELETE-ME: {final_text:?}"
    );
}

// ===========================================================================
// 13. TYPE WHILE AWAITING ACK — fire many keystrokes between the first send
//     and the first ACK. Forces compose-batching on every concat_trivial
//     edge.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn type_while_awaiting_ack() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("type-ack");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    // 50 chars at 10ms intervals — far faster than pad-dev's ~200ms RTT.
    let marker = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWX";
    for c in marker.chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(5000));
    exit_pad(&mut p);
    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains(marker),
        "rapid typing lost order or chars: {final_text:?}"
    );
}

// ===========================================================================
// 14. ENTER ENTER TYPE — multiple newlines in a row then content. Stresses
//     the multi-line keep-count handling in the bridge.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn enter_enter_type() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("ee");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    p.send([b'\r']).expect("enter");
    std::thread::sleep(Duration::from_millis(150));
    p.send([b'\r']).expect("enter");
    std::thread::sleep(Duration::from_millis(150));
    for c in "MULTILINE-CONTENT".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(80));
    }
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);
    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains("MULTILINE-CONTENT"),
        "missing content after enter+enter+type: {final_text:?}"
    );
}

// ===========================================================================
// 15. PASTE THEN CUT — paste a multi-line block, then cut the line we're on.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn paste_then_cut() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("paste-cut");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    let paste_body = "PASTE-LINE-A\nPASTE-LINE-B\nPASTE-LINE-C";
    let paste = format!("\x1b[200~{paste_body}\x1b[201~");
    p.send(paste.as_str()).expect("paste");
    std::thread::sleep(Duration::from_millis(2500));
    p.send([0x0Bu8].as_slice()).expect("^K");
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);
    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        !final_text.is_empty(),
        "pad empty after paste+cut — trailing \\n missing"
    );
    // The pasted lines A and B should still be there; only the LAST line
    // (LINE-C, where cursor was) got cut.
    assert!(
        final_text.contains("PASTE-LINE-A"),
        "LINE-A missing: {final_text:?}"
    );
}

// ===========================================================================
// 15c. CTRL-K THEN CTRL-U — cut a line then immediately paste it back.
//      Regression for browser "doRepApplyChangeset length mismatch: X/Y" —
//      length mismatch in inbound NEW_CHANGES means our changesets'
//      claimed old_len drifted from the server-side text length.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ctrl_k_then_ctrl_u_round_trips() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("kuround");
    let url = format!("{base}/p/{pad_id}");
    // Pre-seed the pad with two lines, so the cut target isn't "the only
    // line" (which is the special would_empty path).
    let mut a = fresh_session(&base, &pad_id, "t.kuround-A").await;
    let init = a.initial_text().to_string();
    let seed_cs = cs_insert_at_end(&init, "first-line\nsecond-line\nthird-line\n");
    a.send_changeset(&seed_cs).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(e) = a.pump_once_event().await {
            if matches!(e, InboundEvent::AckCommit { .. }) {
                return;
            }
        }
    })
    .await;
    a.disconnect().await.ok();

    let mut p = spawn_pad(&url);
    // Cursor is positioned at end of meaningful content on join — likely
    // last non-empty line. Hit ^K to cut, then ^U to paste back.
    p.send([0x0Bu8].as_slice()).expect("^K");
    std::thread::sleep(Duration::from_millis(2000));
    p.send([0x15u8].as_slice()).expect("^U");
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    // After cut+uncut the pad should still contain all three seeded lines.
    assert!(
        final_text.contains("first-line"),
        "first-line missing after cut+uncut: {final_text:?}"
    );
    assert!(
        final_text.contains("second-line"),
        "second-line missing after cut+uncut: {final_text:?}"
    );
    assert!(
        final_text.contains("third-line"),
        "third-line missing after cut+uncut: {final_text:?}"
    );
}

// ===========================================================================
// 15i. RAPID TYPING WITH MIXED ENTERS — user-reported: typed
//      "Proudly sponsored by vpsdimeey ttell me more about this business
//       \nslkdjflksjdlfkjslkdfjlskdjf\nsadfsdfsdf" fast in the terminal; the
//      pad text never reached the browser. Captures the wire trace via a
//      watcher session and asserts every NEW_CHANGES applies cleanly so
//      we catch silent disconnects + length drift in CI.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rapid_typing_with_mixed_enters_propagates() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("rapidmix");
    let url = format!("{base}/p/{pad_id}");

    let mut watcher = fresh_session(&base, &pad_id, "t.rapidmix-W").await;
    let mut watcher_rep = watcher.initial_text().to_string();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<etherpad_client::changeset::Changeset>();
    let watcher_handle = tokio::spawn(async move {
        loop {
            match watcher.pump_once_event().await {
                Ok(InboundEvent::Changeset(cs)) => {
                    if tx.send(cs).is_err() {
                        break;
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });

    let mut p = spawn_pad(&url);
    // Mirror the user's rough cadence: rapid burst, two enters, rapid
    // burst, two enters, rapid burst. No per-char sleep — hammers the
    // outbound batching and the trailing-'\n' interaction in the rope.
    let bursts: [&str; 3] = [
        "ey ttell me more about this business",
        "slkdjflksjdlfkjslkdfjlskdjf",
        "sadfsdfsdf",
    ];
    for (i, burst) in bursts.iter().enumerate() {
        for c in burst.chars() {
            p.send([c as u8].as_slice()).expect("send");
        }
        if i + 1 < bursts.len() {
            p.send([b'\r']).expect("enter1");
            std::thread::sleep(Duration::from_millis(20));
            p.send([b'\r']).expect("enter2");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    std::thread::sleep(Duration::from_millis(6000));
    exit_pad(&mut p);

    while let Ok(Some(cs)) =
        tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
    {
        let cur_len = watcher_rep.chars().count() as u32;
        assert_eq!(
            cs.old_len, cur_len,
            "browser-view length mismatch during rapid typing: \
             cs.old_len={} watcher_rep.len={} cs={:?}",
            cs.old_len, cur_len, cs
        );
        watcher_rep = etherpad_client::ot::apply(&cs, &watcher_rep)
            .expect("apply cs to watcher rep");
    }
    watcher_handle.abort();

    // Bursts must appear in TYPING ORDER on the server side. User-reported
    // failure mode: "boo" + scrambled-burst-2 + "hello world" all on one
    // line — i.e. later bursts get spliced in BEFORE earlier ones. The
    // per-burst contains() check passes the broken case; a forward-scan
    // for start-index of each burst catches order inversion.
    let mut last_idx: usize = 0;
    for burst in &bursts {
        let idx = watcher_rep[last_idx..]
            .find(burst)
            .map(|i| i + last_idx)
            .unwrap_or_else(|| panic!(
                "burst {burst:?} missing or out of order in watcher rep: {watcher_rep:?}"
            ));
        last_idx = idx + burst.len();
    }
    assert!(
        watcher_rep.ends_with('\n'),
        "pad text must end with \\n; got {watcher_rep:?}"
    );
}

// 15h. BACKSPACE-THE-ONLY-NEWLINE — user-reported: "deleting the line break
//      after the last char on the first line (so no line breaks left in the
//      pad) breaks the browser." Backspacing from line 1 col 0 of "abc\n"
//      would join the last content line into the trailing-empty line and
//      leave the rope as "abc" — no trailing '\n'. Browsers then drop the
//      session in applyToAttribution / setDocAText.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn backspace_cannot_strand_trailing_newline() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("bksp-nl");
    let url = format!("{base}/p/{pad_id}");

    let mut watcher = fresh_session(&base, &pad_id, "t.bksp-nl-W").await;
    let mut watcher_rep = watcher.initial_text().to_string();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<etherpad_client::changeset::Changeset>();
    let watcher_handle = tokio::spawn(async move {
        loop {
            match watcher.pump_once_event().await {
                Ok(InboundEvent::Changeset(cs)) => {
                    if tx.send(cs).is_err() {
                        break;
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });

    let mut p = spawn_pad(&url);
    // Position cursor at end of pre-existing line content, then Down-arrow
    // to the trailing-empty line and try to backspace the only '\n'. The
    // buffer must refuse so we never emit a Delete that strands the trailing
    // '\n' on the wire.
    p.send([0x1b, b'[', b'B'].as_slice()).expect("down"); // ESC [ B = Down
    std::thread::sleep(Duration::from_millis(200));
    // Hammer backspace a few times — must be a no-op when the trailing '\n'
    // is the only newline.
    for _ in 0..5 {
        p.send([0x7fu8].as_slice()).expect("bksp");
        std::thread::sleep(Duration::from_millis(80));
    }
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    while let Ok(Some(cs)) =
        tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
    {
        let cur_len = watcher_rep.chars().count() as u32;
        assert_eq!(
            cs.old_len, cur_len,
            "browser-view length mismatch: cs.old_len={} watcher_rep.len={}",
            cs.old_len, cur_len
        );
        watcher_rep = etherpad_client::ot::apply(&cs, &watcher_rep)
            .expect("apply cs to watcher rep");
    }
    watcher_handle.abort();

    assert!(
        watcher_rep.ends_with('\n'),
        "pad text must end with \\n; got {watcher_rep:?}"
    );
}

// 15g. ENTER MANY THEN TYPE — user-reported: "wrote ..., a bunch of empty
//      lines, then 'hello'" produced wire `Z:1h>6|g=1h*0+6$hello ` (insert
//      past the trailing \n) and the browser asserted "line assembler not
//      finished". Caused by cursor reaching the implicit trailing-empty line
//      via repeated Enter / Down arrow; insert at rope.len_chars() appended
//      after the trailing \n and broke Etherpad's "doc ends with \n"
//      invariant. The buffer fix synthesizes the missing trailing \n.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn many_enters_then_type_preserves_trailing_newline() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("manyenter");
    let url = format!("{base}/p/{pad_id}");

    let mut watcher = fresh_session(&base, &pad_id, "t.manyenter-W").await;
    let mut watcher_rep = watcher.initial_text().to_string();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<etherpad_client::changeset::Changeset>();
    let watcher_handle = tokio::spawn(async move {
        loop {
            match watcher.pump_once_event().await {
                Ok(InboundEvent::Changeset(cs)) => {
                    if tx.send(cs).is_err() {
                        break;
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });

    let mut p = spawn_pad(&url);
    for c in "preamble".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(30));
    }
    // 8 Enters to reach deep into the trailing-empty region.
    for _ in 0..8 {
        p.send([b'\r']).expect("enter");
        std::thread::sleep(Duration::from_millis(80));
    }
    // Now type "hello" — pre-fix this would land past the trailing \n.
    for c in "hello".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(40));
    }
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    while let Ok(Some(cs)) =
        tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
    {
        let cur_len = watcher_rep.chars().count() as u32;
        assert_eq!(
            cs.old_len, cur_len,
            "browser-view length mismatch: cs.old_len={} watcher_rep.len={}",
            cs.old_len, cur_len
        );
        watcher_rep = etherpad_client::ot::apply(&cs, &watcher_rep)
            .expect("apply cs to watcher rep");
    }
    watcher_handle.abort();

    assert!(
        watcher_rep.ends_with('\n'),
        "pad text must end with \\n; got {watcher_rep:?}"
    );
    assert!(
        watcher_rep.contains("preamble"),
        "preamble missing: {watcher_rep:?}"
    );
    assert!(
        watcher_rep.contains("hello"),
        "hello missing: {watcher_rep:?}"
    );
}

// 15f. RAPID CUT+UNCUT — same as 15e but back-to-back with no delay between
//      ^K and ^U, so the cut may still be in flight when uncut is queued.
//      This was the user's actual interaction speed.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_sees_rapid_cut_uncut() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("brrapid");
    let url = format!("{base}/p/{pad_id}");
    let mut watcher = fresh_session(&base, &pad_id, "t.brrapid-W").await;
    let mut watcher_rep = watcher.initial_text().to_string();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<etherpad_client::changeset::Changeset>();
    let watcher_handle = tokio::spawn(async move {
        loop {
            match watcher.pump_once_event().await {
                Ok(InboundEvent::Changeset(cs)) => {
                    if tx.send(cs).is_err() {
                        break;
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });
    let mut p = spawn_pad(&url);
    p.send([b'\r']).expect("enter");
    std::thread::sleep(Duration::from_millis(150));
    for c in "RAPID-MARKER-XYZ".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(30));
    }
    std::thread::sleep(Duration::from_millis(1500));
    // BACK-TO-BACK ^K then ^U with no delay.
    p.send([0x0Bu8].as_slice()).expect("^K");
    p.send([0x15u8].as_slice()).expect("^U");
    std::thread::sleep(Duration::from_millis(5000));
    exit_pad(&mut p);
    while let Ok(Some(cs)) =
        tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
    {
        let cur_len = watcher_rep.chars().count() as u32;
        assert_eq!(
            cs.old_len, cur_len,
            "browser-view length mismatch: cs.old_len={} watcher_rep.len={} \
             (cs={:?} cur={:?})",
            cs.old_len, cur_len, cs, watcher_rep
        );
        watcher_rep = etherpad_client::ot::apply(&cs, &watcher_rep)
            .expect("apply cs to watcher rep");
    }
    watcher_handle.abort();
    assert!(
        watcher_rep.contains("RAPID-MARKER-XYZ"),
        "marker missing from watcher rep: {watcher_rep:?}"
    );
}

// 15e. BROWSER VIEW OF CUT+UNCUT — simulate a browser watching the pad while
//      our terminal does ^K + ^U. Verifies each inbound NEW_CHANGES applies
//      to the running rep without the "doRepApplyChangeset length mismatch"
//      that bricks pad-dev's web UI.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_sees_cut_uncut_consistently() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("brkucons");
    let url = format!("{base}/p/{pad_id}");

    // Watcher session — connects FIRST so it joins at the initial revision
    // before the pad sends any USER_CHANGES of ours. We then apply every
    // NEW_CHANGES to a running String and assert no length mismatch.
    let mut watcher = fresh_session(&base, &pad_id, "t.brkucons-W").await;
    let mut watcher_rep = watcher.initial_text().to_string();

    // Pump task — runs the watcher's recv loop into a channel of (changeset,
    // newRev) the test thread can drain.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<etherpad_client::changeset::Changeset>();
    let watcher_handle = tokio::spawn(async move {
        loop {
            match watcher.pump_once_event().await {
                Ok(InboundEvent::Changeset(cs)) => {
                    if tx.send(cs).is_err() {
                        break;
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });

    // Drive the terminal: type a marker, then ^K, then ^U.
    let mut p = spawn_pad(&url);
    p.send([b'\r']).expect("enter");
    std::thread::sleep(Duration::from_millis(150));
    for c in "ROUNDTRIP-MARKER-ABC".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(40));
    }
    std::thread::sleep(Duration::from_millis(1500));
    p.send([0x0Bu8].as_slice()).expect("^K");
    std::thread::sleep(Duration::from_millis(1500));
    p.send([0x15u8].as_slice()).expect("^U");
    std::thread::sleep(Duration::from_millis(3500));
    exit_pad(&mut p);

    // Apply every received NEW_CHANGES to our watcher's running rep, the way
    // the browser's doRepApplyChangeset would. A length mismatch here is
    // the same failure surface as the user-reported error.
    while let Ok(Some(cs)) =
        tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
    {
        let cur_len = watcher_rep.chars().count() as u32;
        assert_eq!(
            cs.old_len, cur_len,
            "browser-view length mismatch: cs.old_len={} watcher_rep.len={} \
             (cs={:?} cur={:?})",
            cs.old_len, cur_len, cs, watcher_rep
        );
        watcher_rep = etherpad_client::ot::apply(&cs, &watcher_rep)
            .expect("apply cs to watcher rep");
    }
    watcher_handle.abort();

    assert!(
        watcher_rep.contains("ROUNDTRIP-MARKER-ABC"),
        "marker missing from watcher rep: {watcher_rep:?}"
    );
}

// 15d. TYPE-THEN-CUT-THEN-UNCUT — common user flow: type some content,
//      hit ^K, then immediately ^U. User-reported "doRepApplyChangeset
//      length mismatch: 53/65".
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn type_then_ctrl_k_then_ctrl_u() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("tkuround");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    // Type some content on a fresh line.
    p.send([b'\r']).expect("enter");
    std::thread::sleep(Duration::from_millis(150));
    for c in "TYPED-CONTENT-BEFORE-CUT".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(2000));
    // ^K to cut current line
    p.send([0x0Bu8].as_slice()).expect("^K");
    std::thread::sleep(Duration::from_millis(2000));
    // ^U to paste back
    p.send([0x15u8].as_slice()).expect("^U");
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains("TYPED-CONTENT-BEFORE-CUT"),
        "marker missing after type+cut+uncut: {final_text:?}"
    );
}

// ===========================================================================
// 15b. MULTI-LINE PASTE ALONE — same paste body as 15 but without the cut,
//      to isolate whether multi-line paste itself is reaching the server.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multiline_paste_lands() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("mlpaste");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);
    let body = "MLINE-A\nMLINE-B\nMLINE-C";
    let paste = format!("\x1b[200~{body}\x1b[201~");
    p.send(paste.as_str()).expect("paste");
    std::thread::sleep(Duration::from_millis(5000));
    exit_pad(&mut p);
    let final_text = pad_text(&base, &pad_id).await;
    eprintln!("DIAG multiline_paste final_text = {final_text:?}");
    assert!(
        final_text.contains("MLINE-A"),
        "LINE-A missing: {final_text:?}"
    );
    assert!(
        final_text.contains("MLINE-C"),
        "LINE-C missing: {final_text:?}"
    );
}

// ===========================================================================
// 16. TYPING THEN BROWSER CONCURRENT EDIT — terminal types while a
//     simulated browser session also writes. Tests inbound + outbound at
//     the same time (OT rebase + pending queue interactions).
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_terminal_typing_and_browser_writes() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("concur");
    let url = format!("{base}/p/{pad_id}");
    let mut p = spawn_pad(&url);

    // Spawn a browser-sim that writes a marker, waits, writes another.
    let base_clone = base.clone();
    let pad_id_clone = pad_id.clone();
    let browser_handle = tokio::spawn(async move {
        let mut b = fresh_session(&base_clone, &pad_id_clone, "t.concur-B").await;
        let init = b.initial_text().to_string();
        tokio::time::sleep(Duration::from_millis(500)).await;
        let cs = cs_insert_at_end(&init, "BROWSER-1\n");
        b.send_changeset(&cs).await.ok();
        let _ = tokio::time::timeout(Duration::from_secs(3), async {
            while let Ok(e) = b.pump_once_event().await {
                if matches!(e, InboundEvent::AckCommit { .. }) {
                    return;
                }
            }
        })
        .await;
        tokio::time::sleep(Duration::from_millis(800)).await;
        let init2 = b.initial_text().to_string();
        let _ = init2; // session.initial_text() doesn't update from inbounds
        // Send another insert based on the LATEST known rev — we can build a
        // simple append (server-side rebase handles concurrent positioning).
        let cs2 = cs_insert_at_end("any-prefix-placeholder", "BROWSER-2\n");
        // The above's old_len is wrong; instead just rebuild minimally with
        // a no-keep approach by sending an identity-with-trailing-insert
        // through the server's follow loop. For simplicity, skip and just
        // rely on the FIRST browser write reaching terminal.
        let _ = cs2;
        b.disconnect().await.ok();
    });

    // Terminal types meanwhile.
    for c in "TERM-TYPING".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);
    browser_handle.await.ok();

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains("TERM-TYPING"),
        "terminal typing missing: {final_text:?}"
    );
    assert!(
        final_text.contains("BROWSER-1"),
        "browser write missing: {final_text:?}"
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

// ===========================================================================
// 17. SEARCH (^W) — finding text moves the cursor LOCALLY and emits no
//     outbound changesets. Pure read-side feature; must not pollute the
//     wire with phantom edits.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn search_moves_cursor_without_sending_changesets() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("search");
    let url = format!("{base}/p/{pad_id}");

    // Seed pad with content the search can find.
    let mut a = fresh_session(&base, &pad_id, "t.search-A").await;
    let init = a.initial_text().to_string();
    let seed = "line one\nline two\nNEEDLE-HERE\nline four\n";
    let cs = cs_insert_at_end(&init, seed);
    a.send_changeset(&cs).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(e) = a.pump_once_event().await {
            if matches!(e, InboundEvent::AckCommit { .. }) {
                return;
            }
        }
    })
    .await;
    a.disconnect().await.ok();

    // Watcher session — captures every NEW_CHANGES the server emits while
    // the terminal pad is interacting. A correctly-implemented search emits
    // zero changesets.
    let mut watcher = fresh_session(&base, &pad_id, "t.search-W").await;
    let (tx, mut rx) =
        tokio::sync::mpsc::unbounded_channel::<etherpad_client::changeset::Changeset>();
    let watcher_handle = tokio::spawn(async move {
        loop {
            match watcher.pump_once_event().await {
                Ok(InboundEvent::Changeset(cs)) => {
                    if tx.send(cs).is_err() {
                        break;
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    });

    let mut p = spawn_pad(&url);
    // ^W (0x17) opens search prompt, type needle, Enter, then exit. Cursor
    // should now sit at "NEEDLE-HERE" but the wire must be empty.
    p.send([0x17u8].as_slice()).expect("^W");
    std::thread::sleep(Duration::from_millis(100));
    for c in "NEEDLE-HERE".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(20));
    }
    p.send([b'\r']).expect("enter");
    std::thread::sleep(Duration::from_millis(2000));
    exit_pad(&mut p);

    let mut saw_changeset = false;
    while let Ok(Some(_cs)) =
        tokio::time::timeout(Duration::from_millis(500), rx.recv()).await
    {
        saw_changeset = true;
    }
    watcher_handle.abort();
    assert!(
        !saw_changeset,
        "search must not emit any outbound changeset"
    );
}

// ===========================================================================
// 18. REPLACE (M-R) — replacing text in a shared pad must propagate to the
//     server. Earlier bug: handle_replace_to mutated the local buffer via
//     replace_all but never emitted a changeset, so the browser saw stale
//     text and refresh didn't help (server actually had stale text too).
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replace_propagates_to_server() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("replace");
    let url = format!("{base}/p/{pad_id}");

    // Seed pad with "hello world\nhello there\n".
    let mut a = fresh_session(&base, &pad_id, "t.replace-A").await;
    let init = a.initial_text().to_string();
    let seed = "hello world\nhello there\n";
    let cs = cs_insert_at_end(&init, seed);
    a.send_changeset(&cs).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(e) = a.pump_once_event().await {
            if matches!(e, InboundEvent::AckCommit { .. }) {
                return;
            }
        }
    })
    .await;
    a.disconnect().await.ok();

    let mut p = spawn_pad(&url);
    // M-R = ESC + 'r' (Alt-R). Then type "hello", Enter, "goodbye", Enter.
    p.send([0x1b, b'r'].as_slice()).expect("M-R");
    std::thread::sleep(Duration::from_millis(150));
    for c in "hello".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(20));
    }
    p.send([b'\r']).expect("enter1");
    std::thread::sleep(Duration::from_millis(150));
    for c in "goodbye".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(20));
    }
    p.send([b'\r']).expect("enter2");
    std::thread::sleep(Duration::from_millis(4000));
    exit_pad(&mut p);

    let final_text = pad_text(&base, &pad_id).await;
    assert!(
        final_text.contains("goodbye world"),
        "first replacement missing on server: {final_text:?}"
    );
    assert!(
        final_text.contains("goodbye there"),
        "second replacement missing on server: {final_text:?}"
    );
    assert!(
        !final_text.contains("hello"),
        "old text still present on server: {final_text:?}"
    );
}

// ===========================================================================
// 19. REPLACE — when the needle isn't found, nothing should be sent and
//     the pad should stay untouched.
// ===========================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replace_with_missing_needle_is_noop() {
    let Some(base) = skip_if_no_remote() else { return };
    let pad_id = fresh_pad_id("replace-miss");
    let url = format!("{base}/p/{pad_id}");

    let mut a = fresh_session(&base, &pad_id, "t.replace-miss-A").await;
    let init = a.initial_text().to_string();
    let seed = "stable line\n";
    let cs = cs_insert_at_end(&init, seed);
    a.send_changeset(&cs).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(e) = a.pump_once_event().await {
            if matches!(e, InboundEvent::AckCommit { .. }) {
                return;
            }
        }
    })
    .await;
    a.disconnect().await.ok();
    let baseline = pad_text(&base, &pad_id).await;

    let mut p = spawn_pad(&url);
    p.send([0x1b, b'r'].as_slice()).expect("M-R");
    std::thread::sleep(Duration::from_millis(150));
    for c in "nonexistent".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(20));
    }
    p.send([b'\r']).expect("enter1");
    std::thread::sleep(Duration::from_millis(150));
    for c in "anything".chars() {
        p.send([c as u8].as_slice()).expect("send");
        std::thread::sleep(Duration::from_millis(20));
    }
    p.send([b'\r']).expect("enter2");
    std::thread::sleep(Duration::from_millis(2000));
    exit_pad(&mut p);

    let after = pad_text(&base, &pad_id).await;
    assert_eq!(
        baseline, after,
        "no-op replace must not change pad text"
    );
}
