//! End-to-end PTY test driving the real `pad` binary against pad-dev.etherpad.org.
//!
//! Skipped when `PAD_ETHERPAD_BASE` is unset (CI default).
//!
//! Reproduces the exact user-reported scenario:
//!   1. `pad <url>` joins a fresh pad on pad-dev.
//!   2. Send a bracketed-paste sequence over the PTY containing a URL.
//!   3. Wait a beat for the outbound Changeset to land + get ACKed.
//!   4. Exit pad with ^X.
//!   5. Open a fresh PadSession to the same pad and assert the pasted URL
//!      is present in the resulting pad text.

use etherpad_client::Socket;
use etherpad_client::session::{PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use expectrl::{Eof, Expect, spawn};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn paste_url_lands_on_remote() {
    let Ok(base) = std::env::var("PAD_ETHERPAD_BASE") else {
        eprintln!("PAD_ETHERPAD_BASE unset, skipping");
        return;
    };
    let pad_id = format!(
        "pad-rust-pty-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let url = format!("{base}/p/{pad_id}");
    eprintln!("target: {url}");

    // 1. Spawn pad binary pointed at the fresh remote pad.
    let bin = env!("CARGO_BIN_EXE_pad");
    let cmd = format!("{bin} {url}");
    let mut p = spawn(cmd).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(10)));

    // Let the editor enter raw mode, finish handshake, draw first frame.
    std::thread::sleep(Duration::from_millis(1500));

    // 2. Send a bracketed-paste sequence with a distinct marker. crossterm
    //    parses ESC[200~ ... ESC[201~ into Event::Paste(content). pad's
    //    InsertText handler builds ONE Changeset and ships it through the
    //    serialized network task.
    let marker = format!(
        "RUST-PTY-PASTE-{}\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    );
    let paste_seq = format!("\x1b[200~{}\x1b[201~", marker);
    p.send(paste_seq.as_str()).expect("send paste");

    // 3. Give it time to: route through input.rs -> App handler -> bridge ->
    //    OutboundQueue -> network task -> socket -> server. Then server ACK
    //    must roundtrip before we exit (otherwise the network task aborts
    //    before sending).
    std::thread::sleep(Duration::from_millis(3000));

    // 4. Exit. ^X -> dirty prompt (we modified the buffer) -> N (don't save).
    p.send([0x18u8].as_slice()).expect("send ^X"); // ^X
    std::thread::sleep(Duration::from_millis(200));
    p.send("N").expect("send N to dirty prompt");
    let _ = p.expect(Eof);

    // 5. Verify the paste landed on the server by opening a fresh session.
    let cookie = TungsteniteSocket::fetch_pad_cookie(&base, &pad_id)
        .await
        .expect("cookie");
    let mut socket = TungsteniteSocket::new(&base, Some(cookie));
    socket.connect().await.expect("connect verifier");
    let mut sess = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: pad_id.clone(),
            token: "t.pty-verifier".into(),
            protocol_version: 2,
        },
    );
    sess.handshake().await.expect("handshake verifier");
    let pad_text = sess.initial_text().to_string();
    sess.disconnect().await.ok();

    eprintln!(
        "pad text after PTY paste ({} chars):\n{pad_text}",
        pad_text.len()
    );
    assert!(
        pad_text.contains(marker.trim_end()),
        "expected paste marker {:?} in pad text after PTY-driven session, got:\n{pad_text}",
        marker.trim_end()
    );
}
