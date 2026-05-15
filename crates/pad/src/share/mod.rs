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
    pub net_task: JoinHandle<()>,
    pub authors: std::collections::HashSet<String>,
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
