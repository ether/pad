use etherpad_client::changeset::Changeset;
use std::collections::VecDeque;
use tokio::sync::mpsc;

/// Tracks local changesets that have been sent to the server but not yet
/// ACK'd via a matching `ACCEPT_COMMIT` (or echoed back as a `NEW_CHANGES`).
///
/// Used to OT-rebase inbound remote changesets so they apply on top of our
/// pending local edits.
pub struct OutboundQueue {
    sink: mpsc::UnboundedSender<Changeset>,
    pending: VecDeque<Changeset>,
}

impl OutboundQueue {
    pub fn new(sink: mpsc::UnboundedSender<Changeset>) -> Self {
        Self {
            sink,
            pending: VecDeque::new(),
        }
    }

    /// Enqueue + send. The changeset stays in `pending` until ACK'd.
    pub fn send(&mut self, cs: Changeset) -> anyhow::Result<()> {
        self.pending.push_back(cs.clone());
        self.sink
            .send(cs)
            .map_err(|_| anyhow::anyhow!("network task closed"))?;
        Ok(())
    }

    /// Drop the oldest pending changeset (the server ACKed it).
    pub fn ack_one(&mut self) {
        self.pending.pop_front();
    }

    pub fn pending(&self) -> impl Iterator<Item = &Changeset> {
        self.pending.iter()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use etherpad_client::changeset::{Changeset, Op, OpCode};

    fn dummy_insert() -> Changeset {
        Changeset {
            old_len: 0,
            net_delta: 1,
            ops: vec![Op {
                opcode: OpCode::Insert,
                chars: 1,
                lines: 0,
                attribs: vec![],
            }],
            char_bank: "x".into(),
        }
    }

    #[test]
    fn enqueue_and_ack() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut q = OutboundQueue::new(tx);
        q.send(dummy_insert()).unwrap();
        q.send(dummy_insert()).unwrap();
        assert_eq!(q.pending_len(), 2);
        let _ = rx.try_recv();
        let _ = rx.try_recv();
        q.ack_one();
        assert_eq!(q.pending_len(), 1);
        q.ack_one();
        assert_eq!(q.pending_len(), 0);
        q.ack_one();
        assert_eq!(q.pending_len(), 0);
    }
}
