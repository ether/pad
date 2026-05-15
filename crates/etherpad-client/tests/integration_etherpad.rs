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

fn etherpad_base() -> String {
    std::env::var("PAD_ETHERPAD_BASE").unwrap_or_else(|_| "http://localhost:9001".to_string())
}

fn pad_id() -> String {
    std::env::var("PAD_ETHERPAD_PAD_ID")
        .unwrap_or_else(|_| "etherpad-client-integration".to_string())
}

async fn etherpad_reachable(base: &str) -> bool {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build();
    let Ok(c) = c else { return false };
    c.get(base).send().await.is_ok()
}

#[tokio::test]
async fn handshake_against_real_etherpad() {
    if std::env::var("PAD_SKIP_DOCKER").is_ok() {
        eprintln!("PAD_SKIP_DOCKER set, skipping");
        return;
    }
    let base = etherpad_base();
    let pid = pad_id();
    if !etherpad_reachable(&base).await {
        eprintln!("Etherpad not reachable at {base}, skipping");
        return;
    }
    eprintln!("target: {base}/p/{pid}");

    let cookie = TungsteniteSocket::fetch_pad_cookie(&base, &pid)
        .await
        .expect("fetch_pad_cookie");
    eprintln!("cookie: {cookie}");

    let mut socket = TungsteniteSocket::new(&base, Some(cookie));
    socket.connect().await.expect("ws connect");

    let mut session = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: pid.clone(),
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
