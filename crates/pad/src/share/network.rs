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
    let mut new_ops = Vec::new();
    if a_keep_chars > 0 {
        new_ops.push(etherpad_client::changeset::Op {
            opcode: OpCode::Keep,
            chars: a_keep_chars,
            lines: a_keep_lines,
            attribs: vec![],
        });
    }
    if merged_lines > 0 && !merged_bank.ends_with('\n') {
        // Multi-line tail without a trailing '\n' — Etherpad rejects that as
        // a single Insert. Split into [prefix-with-final-\n, mid-line tail].
        let last_nl_byte = merged_bank.rfind('\n').expect("has \\n");
        let prefix = &merged_bank[..last_nl_byte + 1];
        let suffix = &merged_bank[last_nl_byte + 1..];
        let prefix_chars = prefix.chars().count() as u32;
        let suffix_chars = suffix.chars().count() as u32;
        new_ops.push(etherpad_client::changeset::Op {
            opcode: OpCode::Insert,
            chars: prefix_chars,
            lines: merged_lines,
            attribs: vec![],
        });
        new_ops.push(etherpad_client::changeset::Op {
            opcode: OpCode::Insert,
            chars: suffix_chars,
            lines: 0,
            attribs: vec![],
        });
    } else {
        new_ops.push(etherpad_client::changeset::Op {
            opcode: OpCode::Insert,
            chars: merged_chars,
            lines: merged_lines,
            attribs: vec![],
        });
    }
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
                            // Before sending the next queued cs, fold the
                            // ENTIRE queue together via concat_inserts. While
                            // we were awaiting the in-flight ACK, app.rs
                            // could have pushed more cs's that the tail-only
                            // merge couldn't reach (e.g. typing → Enter →
                            // typing, where the Enter broke the chain). The
                            // browser-side DOM render bug compounds with the
                            // number of NEW_CHANGES it has to apply, so
                            // collapsing N queued cs's into 1 wire message
                            // is what keeps the live view in sync for fast
                            // typists.
                            while queue.len() >= 2 {
                                let Some((a, ac)) = queue.pop_front() else { break };
                                let Some((b, bc)) = queue.pop_front() else { break };
                                match concat_inserts(&a, &b) {
                                    Some(merged) => {
                                        queue.push_front((merged, ac + bc));
                                    }
                                    None => {
                                        // Can't merge — Delete in the mix, or
                                        // positions don't chain. Put b back
                                        // and ship `a` next.
                                        queue.push_front((b, bc));
                                        queue.push_front((a, ac));
                                        break;
                                    }
                                }
                            }
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
    fn merges_enter_then_typing_splits_into_two_inserts() {
        // "\n" then "t" → merged bank "\nt" is multi-line but doesn't end
        // with \n. Etherpad rejects a single multi-line Insert without a
        // trailing \n, so concat_inserts splits into prefix-with-\n +
        // single-line-tail. Both ops live in ONE changeset so the wire
        // still carries one NEW_CHANGES.
        let a = cs(29, 1, vec![keep(28, 0), insert(1, 1)], "\n");
        let b = cs(30, 1, vec![keep(29, 0), insert(1, 0)], "t");
        let m = concat_inserts(&a, &b).expect("should merge");
        assert_eq!(m.char_bank, "\nt");
        assert_eq!(m.ops.len(), 3); // Keep + Insert(prefix) + Insert(tail)
        assert!(matches!(m.ops[1].opcode, OpCode::Insert));
        assert_eq!(m.ops[1].chars, 1);
        assert_eq!(m.ops[1].lines, 1);
        assert!(matches!(m.ops[2].opcode, OpCode::Insert));
        assert_eq!(m.ops[2].chars, 1);
        assert_eq!(m.ops[2].lines, 0);
    }

    #[test]
    fn merges_multi_op_a_with_single_op_b() {
        // Iteratively merging "\n" + "t" + "e" — after the first merge, a
        // has [Keep, Insert{1,1,"\n"}, Insert{1,0,"t"}]. Merging that with
        // "e" must still chain and produce a valid combined cs.
        let step1_a = cs(29, 1, vec![keep(28, 0), insert(1, 1)], "\n");
        let step1_b = cs(30, 1, vec![keep(29, 0), insert(1, 0)], "t");
        let step1 = concat_inserts(&step1_a, &step1_b).expect("merge 1");
        let step2_b = cs(31, 1, vec![keep(30, 0), insert(1, 0)], "e");
        let step2 = concat_inserts(&step1, &step2_b).expect("merge 2");
        assert_eq!(step2.char_bank, "\nte");
        assert_eq!(step2.old_len, 29);
        assert_eq!(step2.net_delta, 3);
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
