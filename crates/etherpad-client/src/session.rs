//! High-level pad session lifecycle: CLIENT_READY handshake + incoming pump
//! + outgoing USER_CHANGES.

use crate::changeset::Changeset;
use crate::changeset::parser::parse as parse_changeset;
use crate::changeset::serializer::serialize as serialize_changeset;
use crate::error::{ClientError, Result};
use crate::presence::AuthorId;
use crate::socket::Socket;
use serde_json::{Value, json};

pub struct SessionConfig {
    pub pad_id: String,
    /// Legacy in-message token. Modern Etherpad prefers the HttpOnly `token`
    /// cookie carried on the websocket handshake, but still honours this
    /// field for one more release with a deprecation log.
    pub token: String,
    pub protocol_version: u32,
}

pub struct PadSession {
    socket: Box<dyn Socket>,
    cfg: SessionConfig,
    initial_text: String,
    author_id: AuthorId,
    rev: u32,
}

impl PadSession {
    pub fn new(socket: Box<dyn Socket>, cfg: SessionConfig) -> Self {
        Self {
            socket,
            cfg,
            initial_text: String::new(),
            author_id: AuthorId::new(""),
            rev: 0,
        }
    }

    /// Send CLIENT_READY and block until the server replies with CLIENT_VARS.
    /// Populates `initial_text`, `author_id`, and `rev`.
    pub async fn handshake(&mut self) -> Result<()> {
        let ready = json!({
            "component": "pad",
            "type": "CLIENT_READY",
            "padId": self.cfg.pad_id,
            "sessionID": null,
            "token": self.cfg.token,
            "protocolVersion": self.cfg.protocol_version,
        });
        self.socket.emit("message", ready).await?;

        loop {
            let msg =
                self.socket.recv().await.ok_or_else(|| {
                    ClientError::Protocol("socket closed before CLIENT_VARS".into())
                })?;
            if msg["type"] == "CLIENT_VARS" {
                let data = &msg["data"];
                self.author_id = AuthorId::new(data["userId"].as_str().unwrap_or("").to_string());
                self.initial_text = data["collab_client_vars"]["initialAttributedText"]["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                self.rev = data["collab_client_vars"]["rev"].as_u64().unwrap_or(0) as u32;
                return Ok(());
            }
            if let Some(reason) = msg["disconnect"].as_str() {
                return Err(ClientError::Protocol(format!(
                    "server disconnected during handshake: {reason}"
                )));
            }
            if let Some(s) = msg["accessStatus"].as_str() {
                return Err(ClientError::Protocol(format!("access {s}")));
            }
            // ignore unrelated frames
        }
    }

    pub fn initial_text(&self) -> &str {
        &self.initial_text
    }
    pub fn author_id(&self) -> &AuthorId {
        &self.author_id
    }
    pub fn rev(&self) -> u32 {
        self.rev
    }

    /// Send a local changeset built against the current `rev`.
    pub async fn send_changeset(&mut self, cs: &Changeset) -> Result<()> {
        let payload = json!({
            "component": "pad",
            "type": "USER_CHANGES",
            "data": {
                "baseRev": self.rev,
                "changeset": serialize_changeset(cs),
                "apool": { "numToAttrib": {}, "nextNum": 0 }
            }
        });
        self.socket.emit("message", payload).await
    }

    /// Pump one inbound message through the dispatch loop. Returns Ok(Some(cs))
    /// if a remote changeset was received, Ok(None) if some other message was
    /// processed (presence, ACK, etc.), Err on protocol violation, and Err with
    /// a "closed" protocol error when the socket disconnects.
    pub async fn pump_once(&mut self) -> Result<Option<Changeset>> {
        let msg = self
            .socket
            .recv()
            .await
            .ok_or_else(|| ClientError::Protocol("socket closed".into()))?;
        // Etherpad's COLLABROOM envelope wraps NEW_CHANGES etc.
        let kind = msg["type"].as_str().unwrap_or("");
        match kind {
            "COLLABROOM" => {
                let inner = &msg["data"];
                let inner_type = inner["type"].as_str().unwrap_or("");
                match inner_type {
                    "NEW_CHANGES" => {
                        if let Some(cs_wire) = inner["changeset"].as_str() {
                            let cs = parse_changeset(cs_wire)?;
                            if let Some(rev) = inner["newRev"].as_u64() {
                                self.rev = rev as u32;
                            }
                            return Ok(Some(cs));
                        }
                    }
                    "ACCEPT_COMMIT" => {
                        if let Some(rev) = inner["newRev"].as_u64() {
                            self.rev = rev as u32;
                        }
                    }
                    _ => {}
                }
            }
            "CLIENT_VARS" => {
                // post-handshake re-broadcast; ignore.
            }
            _ => {}
        }
        if let Some(d) = msg["disconnect"].as_str() {
            return Err(ClientError::Protocol(format!("server disconnected: {d}")));
        }
        Ok(None)
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.socket.disconnect().await
    }
}

/// Echo `Value` test helper — pulls a raw frame for diagnostics.
pub async fn recv_raw(session: &mut PadSession) -> Option<Value> {
    session.socket.recv().await
}
