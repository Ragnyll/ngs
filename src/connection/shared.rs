use crate::connection::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;

/// Shorthand for the transmit half of the message channel.
pub type Tx = mpsc::UnboundedSender<String>;

/// Data that is shared between all peers in the chat server.
///
/// This is the set of `Tx` handles for all connected clients. Whenever a
/// message is received from a client, it is broadcasted to all peers by
/// iterating over the `peers` entries and sending a copy of the message on each
/// `Tx`.
pub struct Shared<'a> {
    pub better_peers: HashMap<String, Peer<'a>>,
    pub peers: HashMap<SocketAddr, (Tx, i32)>,
    pub peer_count: i32,
}

impl Default for Shared<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Shared<'a> {
    /// Create a new, empty, instance of `Shared`.
    pub fn new() -> Self {
        Self {
            better_peers: HashMap::new(),
            peers: HashMap::new(),
            peer_count: 0,
        }
    }

    /// Send a `LineCodec` encoded message to every peer, except
    /// for the sender.
    pub async fn broadcast(&mut self, sender_user_id: &str, message: &str) {
        for peer in self.better_peers.iter_mut() {
            // the sendee user_id cannot be the sender user_id
            if *peer.0 != sender_user_id {
                let _ = peer.1.tx.send(message.into());
            }
        }
    }

    /// add the peer to the shared state
    pub fn add_peer(&mut self, peer: Peer<'a>) {
        self.better_peers.insert(String::from(peer.user_id), peer);
    }
}
