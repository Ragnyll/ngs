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
pub struct Shared {
    pub peers: HashMap<SocketAddr, (Tx, i32)>,
    pub peer_count: i32,
}

impl Shared {
    /// Create a new, empty, instance of `Shared`.
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            peer_count: 0,
        }
    }

    /// Send a `LineCodec` encoded message to every peer, except
    /// for the sender.
    pub async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1 .0.send(message.into());
            }
        }
    }

    /// Sends a message to the specific sendee
    pub async fn respond(&mut self, sendee: SocketAddr, message: &str) {
        if let Some(peer) = self.find_peer() {
            peer.send("fuck you".into());
        }

    }

    /// gets the Tx handle of the desired peer
    ///
    /// if the peer cannot be found it returns None
    fn find_peer(&self) -> Option<Tx> {
        None
    }
}
