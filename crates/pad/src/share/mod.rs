pub mod bridge;
pub mod collision;
pub mod inbound;
pub mod network;
pub mod osc52;
pub mod outbound;
pub mod qr;
pub mod scanner;
pub mod url_parse;

use etherpad_client::changeset::Changeset;
use outbound::OutboundQueue;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct ShareState {
    pub pad_id: String,
    pub remote_base: String,
    pub author_id: String,
    pub outbound: OutboundQueue,
    pub inbound_rx: mpsc::UnboundedReceiver<Changeset>,
    pub presence_rx: mpsc::UnboundedReceiver<network::PresenceEvent>,
    pub ack_rx: mpsc::UnboundedReceiver<()>,
    pub net_task: JoinHandle<()>,
    pub authors: std::collections::HashSet<String>,
    /// Set to true once we've surfaced the "network task died" FlashMessage
    /// for the current session — drain runs every tick and we don't want to
    /// keep spamming the message ribbon.
    pub disconnected_notified: bool,
}

impl ShareState {
    pub fn share_url(&self) -> String {
        format!(
            "{}/p/{}",
            self.remote_base.trim_end_matches('/'),
            self.pad_id
        )
    }

    pub fn unshare(self) {
        self.net_task.abort();
    }
}
