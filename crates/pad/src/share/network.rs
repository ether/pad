use etherpad_client::Socket;
use etherpad_client::changeset::Changeset;
use etherpad_client::socket::TungsteniteSocket;
use etherpad_client::{PadSession, SessionConfig};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum PresenceEvent {
    Join {
        author_id: String,
        display_name: Option<String>,
    },
    Leave {
        author_id: String,
    },
}

pub struct NetworkHandles {
    pub outbound_tx: mpsc::UnboundedSender<Changeset>,
    pub inbound_rx: mpsc::UnboundedReceiver<Changeset>,
    pub presence_rx: mpsc::UnboundedReceiver<PresenceEvent>,
    /// One () per ACCEPT_COMMIT. The App uses this to drain its local
    /// OutboundQueue so OT rebases only walk truly-unacked changesets.
    pub ack_rx: mpsc::UnboundedReceiver<()>,
    pub task: tokio::task::JoinHandle<()>,
    pub author_id: String,
    pub rev: u32,
    pub initial_text: String,
}

#[derive(Debug)]
pub struct ConnectError {
    pub message: String,
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConnectError {}

pub async fn connect(remote_base: &str, pad_id: &str) -> Result<NetworkHandles, ConnectError> {
    let cookie = TungsteniteSocket::fetch_pad_cookie(remote_base, pad_id)
        .await
        .map_err(|e| ConnectError {
            message: format!("cookie fetch: {e}"),
        })?;
    let mut socket = TungsteniteSocket::new(remote_base, Some(cookie));
    socket.connect().await.map_err(|e| ConnectError {
        message: format!("ws connect: {e}"),
    })?;

    let mut session = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: pad_id.into(),
            token: "t.pad-client-legacy".into(),
            protocol_version: 2,
        },
    );
    session.handshake().await.map_err(|e| ConnectError {
        message: format!("handshake: {e}"),
    })?;

    let author_id = session.author_id().as_str().to_string();
    let rev = session.rev();
    let initial_text = session.initial_text().to_string();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Changeset>();
    let (inbound_tx, inbound_rx) = mpsc::unbounded_channel::<Changeset>();
    let (presence_tx, presence_rx) = mpsc::unbounded_channel::<PresenceEvent>();
    let (ack_tx, ack_rx) = mpsc::unbounded_channel::<()>();

    // Serialized-send design: maintain a local FIFO of pending changesets.
    // Send the head; mark `awaiting_ack`. When ACCEPT_COMMIT arrives (or the
    // server echoes the changeset as NEW_CHANGES from us), pop and send next.
    // This avoids the rev-race where back-to-back sends with stale baseRev
    // get rejected as 'badChangeset'.
    // Send-loop with batching:
    // - At most ONE changeset in flight awaiting ACK at a time (Etherpad
    //   rejects back-to-back sends with stale baseRev).
    // - Outbounds arriving while awaiting are composed via ot::compose into
    //   `queued_tail`, with `queued_count` tracking how many App-side
    //   pending entries are represented. When ACK arrives:
    //     * drain `in_flight_count` ack signals to the App
    //     * if queued_tail: send it, transfer queued_count -> in_flight_count
    //     * else: idle
    // This converts "RTT per keystroke" into "RTT per ACK roundtrip" — typing
    // at 100chars/sec on a 300ms RTT link sees ~30 chars batched per send.
    let task = tokio::spawn(async move {
        let mut session = session;
        let mut queued_tail: Option<Changeset> = None;
        let mut queued_count: usize = 0;
        let mut in_flight_count: usize = 0;
        let mut awaiting_ack = false;
        loop {
            tokio::select! {
                outbound = outbound_rx.recv() => {
                    let Some(cs) = outbound else { break };
                    if awaiting_ack {
                        queued_tail = Some(match queued_tail.take() {
                            None => cs,
                            Some(tail) => match etherpad_client::ot::compose(&tail, &cs) {
                                Ok(c) => c,
                                Err(_) => {
                                    // Compose failed (cs doesn't chain off
                                    // tail) — drop the old tail and start a
                                    // new tail from this cs. The drop costs
                                    // those edits; in practice this shouldn't
                                    // happen for chained user-typing input.
                                    cs
                                }
                            }
                        });
                        queued_count += 1;
                    } else {
                        if session.send_changeset(&cs).await.is_err() { break; }
                        in_flight_count = 1;
                        awaiting_ack = true;
                    }
                }
                pumped = session.pump_once_event() => {
                    match pumped {
                        Ok(etherpad_client::session::InboundEvent::Changeset(cs)) => {
                            if inbound_tx.send(cs).is_err() { break; }
                        }
                        Ok(etherpad_client::session::InboundEvent::AckCommit { .. }) => {
                            for _ in 0..in_flight_count {
                                let _ = ack_tx.send(());
                            }
                            in_flight_count = 0;
                            awaiting_ack = false;
                            if let Some(cs) = queued_tail.take() {
                                if session.send_changeset(&cs).await.is_err() { break; }
                                in_flight_count = queued_count;
                                queued_count = 0;
                                awaiting_ack = true;
                            }
                        }
                        Ok(etherpad_client::session::InboundEvent::UserJoin { author_id, display_name }) => {
                            let _ = presence_tx.send(PresenceEvent::Join { author_id, display_name });
                        }
                        Ok(etherpad_client::session::InboundEvent::UserLeave { author_id }) => {
                            let _ = presence_tx.send(PresenceEvent::Leave { author_id });
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            }
        }
        let _ = session.disconnect().await;
    });

    Ok(NetworkHandles {
        outbound_tx,
        inbound_rx,
        presence_rx,
        ack_rx,
        task,
        author_id,
        rev,
        initial_text,
    })
}
