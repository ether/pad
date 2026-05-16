use etherpad_client::Socket;
use etherpad_client::changeset::{Changeset, OpCode};
use etherpad_client::socket::TungsteniteSocket;
use etherpad_client::{PadSession, SessionConfig};
use std::collections::VecDeque;
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

/// True if `cs` is a "trivial typing keystroke" — a single Insert with no
/// newlines, no attribute decorations, on top of one leading Keep that may
/// have newlines (capturing the existing-pad-prefix). This is the ONLY shape
/// we batch together: it's safe to concatenate two such changesets into
/// "extended insert at the same position" without OT subtlety.
fn is_trivial_typing(cs: &Changeset) -> bool {
    let mut saw_insert = false;
    for op in &cs.ops {
        match op.opcode {
            OpCode::Keep => {
                if saw_insert {
                    return false;
                }
            }
            OpCode::Insert => {
                if saw_insert {
                    return false;
                }
                if op.lines != 0 || !op.attribs.is_empty() {
                    return false;
                }
                saw_insert = true;
            }
            OpCode::Delete => return false,
        }
    }
    saw_insert
}

/// Concatenate two trivial-typing changesets where `b` extends `a`'s insert
/// at the same logical end. Both must satisfy `is_trivial_typing`. Returns
/// None if they don't chain properly.
fn concat_trivial(a: &Changeset, b: &Changeset) -> Option<Changeset> {
    if !is_trivial_typing(a) || !is_trivial_typing(b) {
        return None;
    }
    if a.new_len() != b.old_len {
        return None;
    }
    // a has shape [Keep(K_a)?, Insert(I_a)]. b has shape [Keep(K_b)?, Insert(I_b)].
    // For "typing forward", K_b = K_a + I_a. Result = [Keep(K_a)?, Insert(I_a + I_b)].
    let a_keep_chars: u32 = a
        .ops
        .iter()
        .filter(|o| matches!(o.opcode, OpCode::Keep))
        .map(|o| o.chars)
        .sum();
    let a_keep_lines: u32 = a
        .ops
        .iter()
        .filter(|o| matches!(o.opcode, OpCode::Keep))
        .map(|o| o.lines)
        .sum();
    let a_insert = a
        .ops
        .iter()
        .find(|o| matches!(o.opcode, OpCode::Insert))?;
    let b_keep_chars: u32 = b
        .ops
        .iter()
        .filter(|o| matches!(o.opcode, OpCode::Keep))
        .map(|o| o.chars)
        .sum();
    let b_insert = b
        .ops
        .iter()
        .find(|o| matches!(o.opcode, OpCode::Insert))?;
    if b_keep_chars != a_keep_chars + a_insert.chars {
        return None; // b inserts somewhere other than right after a's insert
    }
    let mut new_ops = Vec::new();
    if a_keep_chars > 0 {
        new_ops.push(etherpad_client::changeset::Op {
            opcode: OpCode::Keep,
            chars: a_keep_chars,
            lines: a_keep_lines,
            attribs: vec![],
        });
    }
    new_ops.push(etherpad_client::changeset::Op {
        opcode: OpCode::Insert,
        chars: a_insert.chars + b_insert.chars,
        lines: 0,
        attribs: vec![],
    });
    Some(Changeset {
        old_len: a.old_len,
        net_delta: a.net_delta + b.net_delta,
        ops: new_ops,
        char_bank: format!("{}{}", a.char_bank, b.char_bank),
    })
}

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

    // Send-loop with NARROW batching:
    // - At most ONE changeset in flight awaiting ACK (Etherpad rejects back-
    //   to-back sends with stale baseRev).
    // - Pending FIFO of fully-formed changesets to send next.
    // - When a new outbound arrives:
    //     * If the queue's tail is a trivial-typing cs AND the new cs also
    //       is + chains off it (e.g. user typed two more chars), MERGE them
    //       into one extended Insert. This collapses streaks of typing into
    //       one send per ACK roundtrip.
    //     * Otherwise: push as a new entry.
    // Why narrow: full ot::compose has subtle correctness bugs around line
    // counts and a-delete interleaving. The trivial-typing batch handles the
    // 80% case (typing without newlines) without touching the complicated
    // compose paths.
    let task = tokio::spawn(async move {
        let mut session = session;
        let mut queue: VecDeque<(Changeset, usize)> = VecDeque::new();
        let mut in_flight_count: usize = 0;
        let mut awaiting_ack = false;
        loop {
            tokio::select! {
                outbound = outbound_rx.recv() => {
                    let Some(cs) = outbound else { break };
                    // Try to merge into the queue tail.
                    let merged = if let Some((tail_cs, tail_count)) = queue.back_mut() {
                        if let Some(joined) = concat_trivial(tail_cs, &cs) {
                            *tail_cs = joined;
                            *tail_count += 1;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !merged {
                        queue.push_back((cs, 1));
                    }
                    // If nothing's in flight, send the front of the queue.
                    if !awaiting_ack && let Some((next, count)) = queue.pop_front() {
                        if session.send_changeset(&next).await.is_err() { break; }
                        in_flight_count = count;
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
                            if let Some((next, count)) = queue.pop_front() {
                                if session.send_changeset(&next).await.is_err() { break; }
                                in_flight_count = count;
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
