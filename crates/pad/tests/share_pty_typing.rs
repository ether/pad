//! Regression: typing a multi-char string character-by-character into a
//! shared pad must produce the string IN ORDER on the server, not reversed.
//!
//! User-reported bug: typing "hello world" appeared in the browser as
//! "dlrow olleh". This test catches whether the wire stream from N
//! individual InsertChar actions lands as the expected forward string on
//! the server.

#![cfg(unix)]
// Unix-PTY harness (expectrl + `env`); self-skips without PAD_ETHERPAD_BASE.

use etherpad_client::Socket;
use etherpad_client::session::{PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use expectrl::{Eof, Expect, spawn};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn typing_chars_lands_forward_not_reversed() {
    let Ok(base) = std::env::var("PAD_ETHERPAD_BASE") else {
        eprintln!("PAD_ETHERPAD_BASE unset, skipping");
        return;
    };
    let pad_id = format!(
        "pad-rust-typing-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let url = format!("{base}/p/{pad_id}");
    eprintln!("target: {url}");

    let bin = env!("CARGO_BIN_EXE_pad");
    let mut p = spawn(format!("{bin} {url}")).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(10)));

    // Let the handshake complete.
    std::thread::sleep(Duration::from_millis(1500));

    // Type one char at a time, pacing to mimic a slow human (and avoiding
    // any rate-limit window).
    let marker = format!(
        "TYP{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
            % 100000
    );
    for c in marker.chars() {
        p.send([c as u8].as_slice()).expect("send char");
        std::thread::sleep(Duration::from_millis(150));
    }

    // Give the network task time to drain its serialized pending queue.
    std::thread::sleep(Duration::from_millis(3000));

    // Exit.
    p.send([0x18u8].as_slice()).expect("send ^X");
    std::thread::sleep(Duration::from_millis(200));
    p.send("N").expect("send N");
    let _ = p.expect(Eof);

    // Open a fresh session and verify the chars landed IN ORDER.
    let cookie = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie");
    let mut socket = TungsteniteSocket::new(&base, Some(cookie));
    socket.connect().await.expect("connect verifier");
    let mut sess = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.typing-verifier".into(),
            protocol_version: 2,
        },
    );
    sess.handshake().await.expect("handshake verifier");
    let pad_text = sess.initial_text().to_string();
    sess.disconnect().await.ok();

    eprintln!("expected forward marker: {marker:?}");
    eprintln!("pad text ({} chars):\n{pad_text}", pad_text.len());

    // The reverse of the marker — if the bug regressed we'd see this.
    let reversed: String = marker.chars().rev().collect();

    assert!(
        pad_text.contains(&marker),
        "expected typed marker {marker:?} (in order) in pad text, got:\n{pad_text}\n\
         (if pad text contains {reversed:?} the per-char insert-pos bug has regressed)"
    );
    assert!(
        !pad_text.contains(&reversed) || marker == reversed,
        "pad text contains reversed marker {reversed:?} — the per-char insert-pos bug has regressed:\n{pad_text}"
    );
}
