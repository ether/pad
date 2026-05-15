//! Hand-rolled socket.io v4 spike against Etherpad.
//!
//! Goal: prove that a tightly-controlled tokio_tungstenite client CAN deliver
//! CLIENT_READY to a modern Etherpad server (where rust_socketio 0.6 failed
//! silently). If this works, we keep hand-rolling for the real client.
//!
//! Protocol notes (Engine.IO v4 + Socket.IO v5 over websocket):
//!   - Connect via `ws://host/socket.io/?EIO=4&transport=websocket`.
//!   - Server immediately sends engine.io OPEN frame: `0{...sid, pingInterval, ...}`.
//!   - Client sends engine.io+socket.io CONNECT: `40` (no namespace = "/").
//!   - Server replies `40{"sid":"..."}` (socket.io connect ack).
//!   - Application events: `42["message", { ...payload... }]`.
//!   - Pings: server sends `2`; client must reply `3`.
//!
//! Modern Etherpad expects the HttpOnly `token=t.xxx` cookie (set by
//! `GET /p/<padid>`) carried on the socket.io handshake — the legacy
//! in-message `token` field is honoured but logs a deprecation warning.

use anyhow::{Context, bail};
use futures_util::{SinkExt, StreamExt};
use http::Request;
use serde_json::{Value, json};
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

const ETHERPAD_HOST: &str = "localhost:9001";
const PAD_ID: &str = "spike-target";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Step 1: fetch the HttpOnly token cookie via GET /p/<padid>.
    let http = reqwest::Client::builder()
        .cookie_store(true)
        .build()?;
    let pad_url = format!("http://{ETHERPAD_HOST}/p/{PAD_ID}");
    eprintln!("[spike] GET {pad_url}");
    let resp = http.get(&pad_url).send().await.context("GET /p/<padid>")?;
    let cookie_header = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|h| h.to_str().ok())
        .find(|s| s.starts_with("token=") || s.starts_with("express.sid="))
        .map(|s| s.split(';').next().unwrap().to_string())
        .context("no token cookie in response")?;
    eprintln!("[spike] cookie: {cookie_header}");

    // Step 2: open websocket directly. EIO=4, transport=websocket skips polling.
    let ws_url = format!("ws://{ETHERPAD_HOST}/socket.io/?EIO=4&transport=websocket");
    eprintln!("[spike] WS connect {ws_url}");
    let request = Request::builder()
        .uri(&ws_url)
        .header("Host", ETHERPAD_HOST)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .header("Cookie", &cookie_header)
        .body(())?;
    let (mut ws, _resp) = tokio_tungstenite::connect_async(request)
        .await
        .context("ws connect")?;
    eprintln!("[spike] websocket connected");

    // Step 3: read engine.io OPEN frame.
    let open_frame = recv_text(&mut ws).await.context("OPEN frame")?;
    eprintln!("[spike] <- {}", truncate(&open_frame, 120));
    if !open_frame.starts_with('0') {
        bail!("expected OPEN frame starting with '0', got: {open_frame}");
    }
    let open_json: Value = serde_json::from_str(&open_frame[1..]).context("parse OPEN")?;
    let sid = open_json["sid"].as_str().context("sid in OPEN")?;
    eprintln!("[spike] sid={sid}");

    // Step 4: send socket.io CONNECT (root namespace).
    eprintln!("[spike] -> 40");
    ws.send(Message::text("40")).await?;

    // Step 5: wait for socket.io connect ack (`40{...}`).
    let connect_ack = recv_text(&mut ws).await.context("CONNECT ack")?;
    eprintln!("[spike] <- {}", truncate(&connect_ack, 120));
    if !connect_ack.starts_with("40") {
        bail!("expected CONNECT ack `40{{...}}`, got: {connect_ack}");
    }

    // Step 6: send CLIENT_READY as a socket.io EVENT (`42["message", {...}]`).
    let client_ready_payload = json!({
        "component": "pad",
        "type": "CLIENT_READY",
        "padId": PAD_ID,
        "sessionID": null,
        "token": "t.spike-token-legacy",
        "protocolVersion": 2,
    });
    let event_frame = format!("42{}", json!(["message", client_ready_payload]));
    eprintln!("[spike] -> {}", truncate(&event_frame, 200));
    ws.send(Message::text(event_frame)).await?;

    // Step 7: read until we see a CLIENT_VARS or an explicit reject, or time out.
    let mut deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let remaining = deadline
            .checked_duration_since(tokio::time::Instant::now())
            .unwrap_or_default();
        if remaining.is_zero() {
            bail!("timeout waiting for CLIENT_VARS");
        }
        match tokio::time::timeout(remaining, ws.next()).await {
            Err(_) => bail!("timeout"),
            Ok(None) => bail!("websocket closed"),
            Ok(Some(Err(e))) => bail!("ws read error: {e}"),
            Ok(Some(Ok(Message::Ping(p)))) => {
                ws.send(Message::Pong(p)).await?;
                continue;
            }
            Ok(Some(Ok(Message::Pong(_)))) => continue,
            Ok(Some(Ok(Message::Close(_)))) => bail!("server closed"),
            Ok(Some(Ok(Message::Binary(_)))) => continue,
            Ok(Some(Ok(Message::Frame(_)))) => continue,
            Ok(Some(Ok(Message::Text(t)))) => {
                let t = t.to_string();
                eprintln!("[spike] <- {}", truncate(&t, 200));
                // Engine.IO ping (`2`) — reply pong (`3`).
                if t == "2" {
                    ws.send(Message::text("3")).await?;
                    continue;
                }
                // Socket.IO EVENT prefix is `42`.
                if let Some(rest) = t.strip_prefix("42") {
                    let arr: Value = serde_json::from_str(rest)?;
                    // arr is ["eventName", arg1, ...]
                    if let Some(arg) = arr.get(1) {
                        let kind = arg["type"].as_str().unwrap_or("");
                        eprintln!("[spike] event type={kind}");
                        if kind == "CLIENT_VARS" {
                            println!("SPIKE OK — received CLIENT_VARS");
                            ws.close(None).await.ok();
                            return Ok(());
                        }
                        if let Some(d) = arg["disconnect"].as_str() {
                            bail!("server disconnected us: {d}");
                        }
                        if let Some(s) = arg["accessStatus"].as_str() {
                            bail!("access denied: {s}");
                        }
                    }
                    // refresh deadline whenever we get *any* event
                    deadline = tokio::time::Instant::now() + Duration::from_secs(10);
                }
            }
        }
    }
}

async fn recv_text<S>(ws: &mut S) -> anyhow::Result<String>
where
    S: futures_util::StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Unpin,
{
    loop {
        match ws.next().await {
            None => bail!("ws closed before frame"),
            Some(Err(e)) => bail!("ws error: {e}"),
            Some(Ok(Message::Text(t))) => return Ok(t.to_string()),
            Some(Ok(_)) => continue,
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max).collect();
        format!("{head}…[{} chars]", s.chars().count())
    }
}
