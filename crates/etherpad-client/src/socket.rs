//! Socket abstraction + a hand-rolled socket.io v4 / engine.io v4 client over
//! tokio-tungstenite. The trait exists so tests can swap in a mock; the real
//! impl in `tungstenite_socket` is what `pad` ships against a real Etherpad.

use crate::error::{ClientError, Result};
use async_trait::async_trait;
use serde_json::Value;

/// A simplified socket.io transport.
///
/// `emit("message", v)` corresponds to a JS-side `socket.emit("message", v)`.
/// `recv()` returns the next inbound socket.io EVENT *argument* — i.e. for an
/// incoming `42["message", { ...payload... }]` we yield `{ ...payload... }`.
#[async_trait]
pub trait Socket: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn emit(&mut self, event: &str, payload: Value) -> Result<()>;
    /// Returns the next inbound message payload, or `None` on disconnect.
    async fn recv(&mut self) -> Option<Value>;
    async fn disconnect(&mut self) -> Result<()>;
}

/// In-memory mock for unit tests.
#[cfg(any(test, feature = "mock-socket"))]
pub mod mock {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::{Mutex, mpsc};

    pub struct MockSocket {
        pub sent: Arc<Mutex<Vec<(String, Value)>>>,
        inbox: mpsc::UnboundedReceiver<Value>,
        pub injector: mpsc::UnboundedSender<Value>,
    }

    impl MockSocket {
        pub fn new() -> Self {
            let (tx, rx) = mpsc::unbounded_channel();
            Self {
                sent: Arc::new(Mutex::new(Vec::new())),
                inbox: rx,
                injector: tx,
            }
        }
    }

    impl Default for MockSocket {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl Socket for MockSocket {
        async fn connect(&mut self) -> Result<()> {
            Ok(())
        }
        async fn emit(&mut self, event: &str, payload: Value) -> Result<()> {
            self.sent.lock().await.push((event.to_string(), payload));
            Ok(())
        }
        async fn recv(&mut self) -> Option<Value> {
            self.inbox.recv().await
        }
        async fn disconnect(&mut self) -> Result<()> {
            Ok(())
        }
    }
}

// -- real impl --------------------------------------------------------------

pub mod tungstenite_socket {
    use super::*;
    use futures_util::stream::SplitSink;
    use futures_util::{SinkExt, StreamExt};
    use http::Request;
    use std::sync::Arc;
    use tokio::net::TcpStream;
    use tokio::sync::{Mutex, mpsc};
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::handshake::client::generate_key;
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

    type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
    type WsSink = SplitSink<WsStream, Message>;

    /// Tokio-tungstenite implementation of [`Socket`].
    ///
    /// Speaks engine.io v4 + socket.io v4 over a single websocket. Pings from
    /// the server are auto-pong'd by a background reader task. Inbound socket.io
    /// EVENT payloads (the second array element of `42["evt", payload]`) are
    /// forwarded over an mpsc channel.
    pub struct TungsteniteSocket {
        url: String,
        cookie: Option<String>,
        writer: Option<Arc<Mutex<WsSink>>>,
        inbox: Option<mpsc::UnboundedReceiver<Value>>,
        reader_task: Option<tokio::task::JoinHandle<()>>,
    }

    impl TungsteniteSocket {
        /// `base_url` should be `http://host:port` or `https://host:port` —
        /// this is what `GET /p/<padid>` is hit against and the same origin is
        /// used (over `ws://` / `wss://`) for the websocket. The optional
        /// `cookie` header (e.g. `token=t.xxx`) is sent on the websocket
        /// handshake.
        pub fn new(base_url: impl Into<String>, cookie: Option<String>) -> Self {
            Self {
                url: base_url.into(),
                cookie,
                writer: None,
                inbox: None,
                reader_task: None,
            }
        }

        /// Convenience: fetch `GET <base>/p/<pad_id>` over HTTP and harvest the
        /// HttpOnly `token=` cookie from the response. Needed for modern
        /// Etherpad (post-GDPR-PR3) which validates that cookie on the
        /// socket.io handshake.
        pub async fn fetch_pad_cookie(base_url: &str, pad_id: &str) -> Result<String> {
            let url = format!("{}/p/{}", base_url.trim_end_matches('/'), pad_id);
            let client = reqwest::Client::builder()
                .build()
                .map_err(|e| ClientError::Socket(format!("reqwest build: {e}")))?;
            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| ClientError::Socket(format!("GET {url}: {e}")))?;
            for h in resp.headers().get_all("set-cookie").iter() {
                let s = h
                    .to_str()
                    .map_err(|e| ClientError::Socket(format!("bad cookie hdr: {e}")))?;
                let kv = s.split(';').next().unwrap_or(s);
                if kv.starts_with("token=") {
                    return Ok(kv.to_string());
                }
            }
            Err(ClientError::Socket(format!(
                "no token cookie in response from {url}"
            )))
        }
    }

    #[async_trait]
    impl Socket for TungsteniteSocket {
        async fn connect(&mut self) -> Result<()> {
            let ws_url = self
                .url
                .replacen("https://", "wss://", 1)
                .replacen("http://", "ws://", 1);
            let ws_url = format!(
                "{}/socket.io/?EIO=4&transport=websocket",
                ws_url.trim_end_matches('/')
            );

            let parsed = url::Url::parse(&ws_url)
                .map_err(|e| ClientError::Socket(format!("bad url: {e}")))?;
            let host = parsed
                .host_str()
                .ok_or_else(|| ClientError::Socket("url has no host".into()))?;
            let host_header = match parsed.port() {
                Some(p) => format!("{host}:{p}"),
                None => host.to_string(),
            };

            let mut builder = Request::builder()
                .uri(&ws_url)
                .header("Host", &host_header)
                .header("Upgrade", "websocket")
                .header("Connection", "Upgrade")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", generate_key());
            if let Some(c) = &self.cookie {
                builder = builder.header("Cookie", c);
            }
            let request = builder
                .body(())
                .map_err(|e| ClientError::Socket(format!("build request: {e}")))?;

            let (ws, _resp) = connect_async(request)
                .await
                .map_err(|e| ClientError::Socket(format!("ws connect: {e}")))?;
            let (sink, mut stream) = ws.split();

            // Engine.IO OPEN frame.
            let open = expect_text(&mut stream).await?;
            if !open.starts_with('0') {
                return Err(ClientError::Protocol(format!(
                    "expected OPEN frame, got: {open}"
                )));
            }

            // Send socket.io CONNECT and read its ack on the same (unsplit) flow.
            let writer = Arc::new(Mutex::new(sink));
            {
                let mut w = writer.lock().await;
                w.send(Message::text("40"))
                    .await
                    .map_err(|e| ClientError::Socket(format!("send CONNECT: {e}")))?;
            }
            let connect_ack = expect_text(&mut stream).await?;
            if !connect_ack.starts_with("40") {
                return Err(ClientError::Protocol(format!(
                    "expected CONNECT ack, got: {connect_ack}"
                )));
            }

            // Spawn the reader.
            let (inbox_tx, inbox_rx) = mpsc::unbounded_channel::<Value>();
            let writer_for_reader = writer.clone();
            let reader_task = tokio::spawn(async move {
                while let Some(msg) = stream.next().await {
                    let frame = match msg {
                        Ok(Message::Text(t)) => t.to_string(),
                        Ok(Message::Ping(p)) => {
                            let mut w = writer_for_reader.lock().await;
                            let _ = w.send(Message::Pong(p)).await;
                            continue;
                        }
                        Ok(Message::Close(_)) => break,
                        Ok(_) => continue,
                        Err(_) => break,
                    };
                    // Engine.IO ping frame (`2`) — respond pong (`3`).
                    if frame == "2" {
                        let mut w = writer_for_reader.lock().await;
                        let _ = w.send(Message::text("3")).await;
                        continue;
                    }
                    // Socket.IO EVENT frame: `42[...]`.
                    if let Some(rest) = frame.strip_prefix("42") {
                        if std::env::var("PAD_SOCKET_DEBUG").is_ok() {
                            eprintln!(
                                "[socket] <- 42{}",
                                if rest.len() > 200 { &rest[..200] } else { rest }
                            );
                        }
                        if let Ok(arr) = serde_json::from_str::<Value>(rest)
                            && let Some(payload) = arr.get(1).cloned()
                        {
                            let _ = inbox_tx.send(payload);
                        }
                        continue;
                    }
                    if std::env::var("PAD_SOCKET_DEBUG").is_ok() {
                        eprintln!("[socket] <- {frame}");
                    }
                    // Server-initiated DISCONNECT `41`: close the reader.
                    if frame == "41" {
                        break;
                    }
                }
            });

            self.writer = Some(writer);
            self.inbox = Some(inbox_rx);
            self.reader_task = Some(reader_task);
            Ok(())
        }

        async fn emit(&mut self, _event: &str, payload: Value) -> Result<()> {
            // Etherpad only uses the implicit "message" event for all
            // socket.io traffic. We ignore `event` and always emit
            // `42["message", payload]`. The trait method keeps the JS-style
            // `emit("event", payload)` signature for callers that might
            // someday want a different namespace.
            let frame = format!(
                "42{}",
                Value::Array(vec![Value::String("message".to_string()), payload])
            );
            let writer = self
                .writer
                .as_ref()
                .ok_or_else(|| ClientError::Socket("not connected".into()))?;
            let mut w = writer.lock().await;
            if std::env::var("PAD_SOCKET_DEBUG").is_ok() {
                eprintln!(
                    "[socket] -> {}",
                    if frame.len() > 200 {
                        &frame[..200]
                    } else {
                        &frame
                    }
                );
            }
            w.send(Message::text(frame))
                .await
                .map_err(|e| ClientError::Socket(format!("emit: {e}")))?;
            Ok(())
        }

        async fn recv(&mut self) -> Option<Value> {
            self.inbox.as_mut()?.recv().await
        }

        async fn disconnect(&mut self) -> Result<()> {
            if let Some(w) = &self.writer {
                let mut lock = w.lock().await;
                let _ = lock.send(Message::Close(None)).await;
            }
            if let Some(t) = self.reader_task.take() {
                t.abort();
            }
            self.writer = None;
            self.inbox = None;
            Ok(())
        }
    }

    async fn expect_text<S>(stream: &mut S) -> Result<String>
    where
        S: StreamExt<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + Unpin,
    {
        loop {
            match stream.next().await {
                None => return Err(ClientError::Socket("ws closed".into())),
                Some(Err(e)) => return Err(ClientError::Socket(format!("ws read: {e}"))),
                Some(Ok(Message::Text(t))) => return Ok(t.to_string()),
                Some(Ok(_)) => continue,
            }
        }
    }
}

pub use tungstenite_socket::TungsteniteSocket;
