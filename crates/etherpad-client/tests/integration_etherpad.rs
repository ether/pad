//! End-to-end test: connect a PadSession against a real Etherpad instance.
//!
//! Skipped automatically if `PAD_SKIP_DOCKER=1` is set OR if there is no
//! Etherpad reachable on `http://localhost:9001`. The caller is responsible
//! for spinning up the container (the integration suite in CI does this; for
//! local runs see `spike/README.md`).

use etherpad_client::Socket;
use etherpad_client::session::{PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use std::time::Duration;

const ETHERPAD_BASE: &str = "http://localhost:9001";
const PAD_ID: &str = "etherpad-client-integration";

async fn etherpad_reachable() -> bool {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build();
    let Ok(c) = c else { return false };
    c.get(ETHERPAD_BASE).send().await.is_ok()
}

#[tokio::test]
async fn handshake_against_real_etherpad() {
    if std::env::var("PAD_SKIP_DOCKER").is_ok() {
        eprintln!("PAD_SKIP_DOCKER set, skipping");
        return;
    }
    if !etherpad_reachable().await {
        eprintln!("Etherpad not reachable on :9001, skipping (set up via spike/README.md)");
        return;
    }

    let cookie = TungsteniteSocket::fetch_pad_cookie(ETHERPAD_BASE, PAD_ID)
        .await
        .expect("fetch_pad_cookie");
    eprintln!("cookie: {cookie}");

    let mut socket = TungsteniteSocket::new(ETHERPAD_BASE, Some(cookie));
    socket.connect().await.expect("ws connect");

    let mut session = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: PAD_ID.into(),
            token: "t.integration-legacy".into(),
            protocol_version: 2,
        },
    );
    let handshake = tokio::time::timeout(Duration::from_secs(10), session.handshake()).await;
    handshake
        .expect("handshake timed out")
        .expect("handshake failed");

    // CLIENT_VARS should have given us an author id and an initial text.
    assert!(
        !session.author_id().as_str().is_empty(),
        "expected an author id from CLIENT_VARS, got empty"
    );
    eprintln!(
        "author={} rev={} initial_text_chars={}",
        session.author_id().as_str(),
        session.rev(),
        session.initial_text().chars().count()
    );
}
