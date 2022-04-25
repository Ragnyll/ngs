use crate::connection::shared::Shared;

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LinesCodec};

/// Shorthand for the receive half of the message channel.
pub type Rx = mpsc::UnboundedReceiver<String>;

/// Shorthand for the transmit half of the message channel.
pub type Tx = mpsc::UnboundedSender<String>;

/// The state for each connected client.
pub struct Peer<'a> {
    /// The TCP socket wrapped with the `Lines` codec, defined below.
    ///
    /// This handles sending and receiving data on the socket. When using
    /// `Lines`, we can work at the line level instead of having to manage the
    /// raw byte operations.
    pub lines: Framed<TcpStream, LinesCodec>,

    /// Receive half of the message channel.
    ///
    /// This is used to receive messages from peers. When a message is received
    /// off of this `Rx`, it will be written to the socket.
    pub rx: Rx,

    /// Transmit half of the message channel.
    ///
    /// This is used to send a message to this peer.
    pub tx: Tx,

    /// The SocketAddress of the peer.
    pub addr: SocketAddr,

    /// The identifier uniquely identifying a user for their connection
    /// TODO: make into a uuid
    pub user_id: &'a str,
}

impl<'a> Peer<'a> {
    /// Create a new instance of `Peer`.
    pub async fn new(
        state: Arc<Mutex<Shared<'_>>>,
        lines: Framed<TcpStream, LinesCodec>,
        user_id: &'a str,
    ) -> io::Result<Peer<'a>> {
        // Get the client socket address
        let addr = lines.get_ref().peer_addr()?;

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        let current_peer = state.lock().await.peer_count;
        state.lock().await.peer_count += 1;
        state
            .lock()
            .await
            .peers
            .insert(addr, (tx.clone(), current_peer));

        Ok(Peer {
            lines,
            rx,
            tx,
            addr,
            user_id,
        })
    }
}
