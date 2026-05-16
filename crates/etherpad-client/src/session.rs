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

/// Typed envelope for inbound socket.io frames.
pub enum InboundEvent {
    /// A remote-authored changeset that should apply to the local pad.
    Changeset(Changeset),
    /// Server accepted the most recent `send_changeset` and bumped `rev` to
    /// `new_rev`. The session's internal `rev` is already updated by the time
    /// this event is returned. Senders must wait for this before issuing the
    /// next `send_changeset` — Etherpad will reject a back-to-back send whose
    /// `baseRev` matches the older rev but whose `old_len` reflects the new
    /// (locally-applied) length.
    AckCommit { new_rev: u32 },
    /// A peer joined the pad. May fire on every USER_NEWINFO heartbeat too —
    /// callers should treat repeated joins as idempotent.
    UserJoin {
        author_id: String,
        display_name: Option<String>,
    },
    /// A peer left the pad.
    UserLeave { author_id: String },
    /// Anything else — typed presence events, presence beats, server hints.
    Other,
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
    ///
    /// USER_CHANGES is dispatched as a COLLABROOM envelope (see
    /// PadMessageHandler.ts line 522) — top-level `type: USER_CHANGES` would
    /// be silently ignored.
    pub async fn send_changeset(&mut self, cs: &Changeset) -> Result<()> {
        let payload = json!({
            "component": "pad",
            "type": "COLLABROOM",
            "data": {
                "type": "USER_CHANGES",
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

    /// Like `pump_once`, but returns a typed `InboundEvent` so callers can
    /// distinguish changesets, presence joins, and presence leaves. Used by
    /// `pad`'s share-network task.
    pub async fn pump_once_event(&mut self) -> Result<InboundEvent> {
        let msg = self
            .socket
            .recv()
            .await
            .ok_or_else(|| ClientError::Protocol("socket closed".into()))?;
        let kind = msg["type"].as_str().unwrap_or("");
        if kind == "COLLABROOM" {
            let inner = &msg["data"];
            match inner["type"].as_str().unwrap_or("") {
                "NEW_CHANGES" => {
                    if let Some(wire) = inner["changeset"].as_str() {
                        let cs = parse_changeset(wire)?;
                        if let Some(rev) = inner["newRev"].as_u64() {
                            self.rev = rev as u32;
                        }
                        return Ok(InboundEvent::Changeset(cs));
                    }
                }
                "ACCEPT_COMMIT" => {
                    if let Some(rev) = inner["newRev"].as_u64() {
                        self.rev = rev as u32;
                        return Ok(InboundEvent::AckCommit { new_rev: rev as u32 });
                    }
                }
                "USER_NEWINFO" => {
                    let user_info = &inner["userInfo"];
                    let author_id = user_info["userId"].as_str().unwrap_or("").to_string();
                    let display = user_info["name"].as_str().map(|s| s.to_string());
                    if !author_id.is_empty() {
                        return Ok(InboundEvent::UserJoin {
                            author_id,
                            display_name: display,
                        });
                    }
                }
                "USER_LEAVE" => {
                    let user_info = &inner["userInfo"];
                    let author_id = user_info["userId"].as_str().unwrap_or("").to_string();
                    if !author_id.is_empty() {
                        return Ok(InboundEvent::UserLeave { author_id });
                    }
                }
                _ => {}
            }
        }
        Ok(InboundEvent::Other)
    }
}

/// Echo `Value` test helper — pulls a raw frame for diagnostics.
pub async fn recv_raw(session: &mut PadSession) -> Option<Value> {
    session.socket.recv().await
}
