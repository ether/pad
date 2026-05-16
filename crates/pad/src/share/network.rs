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
/// True when `cs` is a "leading-Keep + one-or-more Insert(s)" shape with no
/// Delete ops and no attribs. Both single-line and multi-line typing-style
/// changesets qualify; the multi-line case can have two Insert ops when a
/// prior compose split a `\n`-terminated prefix off a mid-line tail (see
/// [`concat_inserts`] / [`bridge::changeset_for_insert`]).
fn is_insert_only(cs: &Changeset) -> bool {
    let mut keep_done = false;
    let mut saw_insert = false;
    for op in &cs.ops {
        match op.opcode {
            OpCode::Keep => {
                if saw_insert {
                    return false;
                }
                if !op.attribs.is_empty() {
                    return false;
                }
                keep_done = true;
            }
            OpCode::Insert => {
                let _ = keep_done; // reserved for future shape checks
                if !op.attribs.is_empty() {
                    return false;
                }
                saw_insert = true;
            }
            OpCode::Delete => return false,
        }
    }
    saw_insert
}

/// Concatenate two insert-only changesets where `b` extends `a`'s insert at
/// the same logical end. Handles BOTH single-line typing (the original
/// concat_trivial case) AND multi-line bursts that cross `\n`s — so a full
/// "type → Enter → type → Enter → type" sequence collapses into ONE wire
/// changeset, and the receiving browser does ONE DOM apply instead of N.
///
/// The reason this matters: Etherpad's web client has a DOM-render
/// inconsistency under rapid remote NEW_CHANGES (verified via Node watcher
/// against `applyToAText` — the underlying rep is correct; only the
/// live-rendered DOM scrambles). Sending fewer, larger changesets keeps
/// that bug from triggering for our users.
///
/// Etherpad's `checkRep` requires any Insert op with `lines > 0` to have its
/// char_bank slice END with `\n`. When the merged bank has internal newlines
/// but a mid-line tail (e.g. `"\n\ntesting"`), we emit TWO Insert ops in the
/// returned changeset: a prefix Insert{lines=N, ends with \n} plus a single-
/// line Insert{lines=0} for the tail. Both ops in one changeset → still one
/// NEW_CHANGES on the wire.
fn concat_inserts(a: &Changeset, b: &Changeset) -> Option<Changeset> {
    if !is_insert_only(a) || !is_insert_only(b) {
        return None;
    }
    if a.new_len() != b.old_len {
        return None;
    }
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
    let a_insert_chars: u32 = a
        .ops
        .iter()
        .filter(|o| matches!(o.opcode, OpCode::Insert))
        .map(|o| o.chars)
        .sum();
    let b_keep_chars: u32 = b
        .ops
        .iter()
        .filter(|o| matches!(o.opcode, OpCode::Keep))
        .map(|o| o.chars)
        .sum();
    if b_keep_chars != a_keep_chars + a_insert_chars {
        return None; // b inserts somewhere other than right after a's insert
    }
    let merged_bank = format!("{}{}", a.char_bank, b.char_bank);
    let merged_chars = merged_bank.chars().count() as u32;
    let merged_lines = merged_bank.matches('\n').count() as u32;
    // Refuse to produce a multi-Insert-op changeset. Etherpad's web client
    // never emits that shape (its compose() flushes at line boundaries);
    // observed live: the receiving browser scrambles content in the live
    // DOM when it sees Keep + Insert(lines>0) + Insert(lines=0) in one
    // changeset, even though `applyToAText` produces the correct rep.
    // Keep is fine; multi-Insert is what trips the render path. So when a
    // merge would require splitting into prefix-with-\n + mid-line tail,
    // we DON'T merge — leave the two cs's separate, one wire message
    // each.
    if merged_lines > 0 && !merged_bank.ends_with('\n') {
        return None;
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
        chars: merged_chars,
        lines: merged_lines,
        attribs: vec![],
    });
    Some(Changeset {
        old_len: a.old_len,
        net_delta: a.net_delta + b.net_delta,
        ops: new_ops,
        char_bank: merged_bank,
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
    // Etherpad's web client uses commitDelay = 500ms — it sends at most
    // one USER_CHANGES per 500ms even when keystrokes are arriving faster.
    // We mirror that here so the rate of NEW_CHANGES that downstream
    // browsers must apply matches what they'd see from a peer browser,
    // not 2-3x higher (which trips a known DOM-render scramble in
    // Etherpad's web client under rapid remote NEW_CHANGES — verified out
    // of band via Node-side `applyToAText` watcher).
    let commit_delay = std::time::Duration::from_millis(500);

    let task = tokio::spawn(async move {
        let mut session = session;
        let mut queue: VecDeque<(Changeset, usize)> = VecDeque::new();
        let mut in_flight_count: usize = 0;
        let mut awaiting_ack = false;
        // Earliest moment we're allowed to start the next send.
        // `tokio::time::Instant::now()` initially so the first cs ships
        // immediately; bumped to `now + commit_delay` after each send.
        let mut next_send_allowed = tokio::time::Instant::now();
        loop {
            // Fold the queue first so the deadline-arm sees a single
            // entry and the outbound-arm's tail-merge has the chance to
            // catch every chain.
            while queue.len() >= 2 {
                let Some((a, ac)) = queue.pop_front() else { break };
                let Some((b, bc)) = queue.pop_front() else { break };
                match concat_inserts(&a, &b) {
                    Some(merged) => {
                        queue.push_front((merged, ac + bc));
                    }
                    None => {
                        queue.push_front((b, bc));
                        queue.push_front((a, ac));
                        break;
                    }
                }
            }

            // Deadline future — only "armed" when there's something to
            // send AND we're not blocked on an ACK. Otherwise sleep for a
            // long no-op so the select! still has 3 arms.
            let ready_to_send = !awaiting_ack && !queue.is_empty();
            let deadline = if ready_to_send {
                next_send_allowed
            } else {
                tokio::time::Instant::now() + std::time::Duration::from_secs(3600)
            };

            tokio::select! {
                outbound = outbound_rx.recv() => {
                    let Some(cs) = outbound else { break };
                    // Try to merge into the queue tail.
                    let merged = if let Some((tail_cs, tail_count)) = queue.back_mut() {
                        if let Some(joined) = concat_inserts(tail_cs, &cs) {
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
                    // Note: we deliberately DO NOT send eagerly here. The
                    // next-send-allowed deadline gates dispatch so chars
                    // typed within `commit_delay` of the previous send get
                    // a chance to merge into the same wire message.
                }
                _ = tokio::time::sleep_until(deadline), if ready_to_send => {
                    if let Some((next, count)) = queue.pop_front() {
                        if session.send_changeset(&next).await.is_err() { break; }
                        in_flight_count = count;
                        awaiting_ack = true;
                        next_send_allowed = tokio::time::Instant::now() + commit_delay;
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
                            // Don't send immediately — let the deadline
                            // arm fire so commit_delay is honored across
                            // ACK boundaries too. next_send_allowed is
                            // already set to (last_send_time + delay), so
                            // the deadline-arm will fire as soon as the
                            // delay window elapses.
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

#[cfg(test)]
mod concat_tests {
    use super::*;
    use etherpad_client::changeset::{Changeset, Op, OpCode};

    fn keep(chars: u32, lines: u32) -> Op {
        Op {
            opcode: OpCode::Keep,
            chars,
            lines,
            attribs: vec![],
        }
    }
    fn insert(chars: u32, lines: u32) -> Op {
        Op {
            opcode: OpCode::Insert,
            chars,
            lines,
            attribs: vec![],
        }
    }
    fn cs(old_len: u32, net: i64, ops: Vec<Op>, bank: &str) -> Changeset {
        Changeset {
            old_len,
            net_delta: net,
            ops,
            char_bank: bank.into(),
        }
    }

    #[test]
    fn merges_single_line_typing() {
        // "h" at pos 28 of 29-char doc, then "e" at pos 29 of 30-char doc.
        let a = cs(29, 1, vec![keep(28, 0), insert(1, 0)], "h");
        let b = cs(30, 1, vec![keep(29, 0), insert(1, 0)], "e");
        let m = concat_inserts(&a, &b).expect("should merge");
        assert_eq!(m.old_len, 29);
        assert_eq!(m.net_delta, 2);
        assert_eq!(m.char_bank, "he");
        assert_eq!(m.ops.len(), 2);
        assert!(matches!(m.ops[0].opcode, OpCode::Keep));
        assert_eq!(m.ops[0].chars, 28);
        assert!(matches!(m.ops[1].opcode, OpCode::Insert));
        assert_eq!(m.ops[1].chars, 2);
        assert_eq!(m.ops[1].lines, 0);
    }

    #[test]
    fn merges_two_enters_into_one_multiline_insert() {
        // Two single-\n inserts that chain — result is "\n\n" (multi-line,
        // ends with \n) and stays a single Insert op.
        let a = cs(29, 1, vec![keep(28, 0), insert(1, 1)], "\n");
        let b = cs(30, 1, vec![keep(29, 0), insert(1, 1)], "\n");
        let m = concat_inserts(&a, &b).expect("should merge");
        assert_eq!(m.char_bank, "\n\n");
        assert_eq!(m.ops.len(), 2);
        assert_eq!(m.ops[1].chars, 2);
        assert_eq!(m.ops[1].lines, 2);
    }

    #[test]
    fn refuses_to_merge_enter_then_mid_line_typing() {
        // "\n" then "t" → merged bank "\nt" is multi-line but doesn't end
        // with '\n'. Producing that as a single Insert violates Etherpad's
        // checkRep, and producing it as two Insert ops in one changeset
        // triggers a DOM-render scramble in Etherpad's web client (lives
        // in performDocumentApplyChangeset → DOM splice). Match the
        // browser's commit shape instead — leave the two cs's separate.
        let a = cs(29, 1, vec![keep(28, 0), insert(1, 1)], "\n");
        let b = cs(30, 1, vec![keep(29, 0), insert(1, 0)], "t");
        assert!(concat_inserts(&a, &b).is_none());
    }

    #[test]
    fn merges_typing_then_enter_into_single_multiline_insert() {
        // "hello" + "\n" → merged bank "hello\n" ends with '\n' → one
        // Insert op (lines=1) — this is the shape Etherpad's web client
        // also produces for a typing→Enter burst, and rendering handles
        // it correctly.
        let a = cs(29, 5, vec![keep(28, 0), insert(5, 0)], "hello");
        let b = cs(34, 1, vec![keep(33, 0), insert(1, 1)], "\n");
        let m = concat_inserts(&a, &b).expect("should merge");
        assert_eq!(m.char_bank, "hello\n");
        assert_eq!(m.ops.len(), 2); // Keep + single Insert
        assert!(matches!(m.ops[1].opcode, OpCode::Insert));
        assert_eq!(m.ops[1].chars, 6);
        assert_eq!(m.ops[1].lines, 1);
    }

    #[test]
    fn refuses_to_merge_when_chain_broken() {
        // b's keep doesn't line up with end of a's insert — must refuse.
        let a = cs(29, 1, vec![keep(28, 0), insert(1, 0)], "h");
        let b = cs(30, 1, vec![keep(5, 0), insert(1, 0)], "x");
        assert!(concat_inserts(&a, &b).is_none());
    }

    #[test]
    fn refuses_to_merge_when_b_contains_delete() {
        let a = cs(29, 1, vec![keep(28, 0), insert(1, 0)], "h");
        let b = cs(30, -1, vec![keep(29, 0), Op { opcode: OpCode::Delete, chars: 1, lines: 0, attribs: vec![] }], "");
        assert!(concat_inserts(&a, &b).is_none());
    }
}
