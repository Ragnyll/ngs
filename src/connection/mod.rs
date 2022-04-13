mod handshake;

use crate::room;
use futures::SinkExt;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LinesCodec};

/// Shorthand for the transmit half of the message channel.
type Tx = mpsc::UnboundedSender<String>;

/// Shorthand for the receive half of the message channel.
type Rx = mpsc::UnboundedReceiver<String>;

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

    /// The room id that the peer belongs to.
    pub room_id: u32,

    pub user_id: &'a str,
}

impl<'a> Peer<'a> {
    /// Create a new instance of `Peer`.
    pub async fn new(
        state: Arc<Mutex<Shared>>,
        lines: Framed<TcpStream, LinesCodec>,
        room_id: u32,
    ) -> io::Result<Peer<'a>> {
        // Get the client socket address
        let addr = lines.get_ref().peer_addr()?;

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        let current_peer = state.lock().await.peer_count;
        state.lock().await.peer_count += 1;
        state.lock().await.peers.insert(addr, (tx, current_peer));

        Ok(Peer {
            lines,
            rx,
            room_id,
            user_id: "billy",
        })
    }
}

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
                //  && peer.1.1 == 1 {
                let _ = peer.1 .0.send(message.into());
            }
        }
    }
}

/// Process an individual chat client
pub async fn process(
    room_meta: Arc<Mutex<room::RoomMeta>>,
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    log::info!("Received connection request, awaiting connection info");
    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            // TODO handle the connection message here
            // TODO timeout a connection if not received the connection string
            println!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(());
        }
    };

    lines
        .send("Please enter the user_id of whom you want to pair with")
        .await?;
    let requested_user_id: u32 = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            println!(
                "Failed to get user_id_request from {}. Client disconnected.",
                addr
            );
            return Ok(());
        }
    }
    .parse()
    .unwrap();

    // // if requested user_id is found then get the room needed, otherwise create a new room
    // let room_id = match room::find_room_with_user(requested_user_id, &room_meta).await {
    // Some(r) => {
    // println!("found the user in room {r}. Lemme match them up");
    // room::assign_user_to_room(username.parse::<u32>().unwrap(), r, &room_meta).await;
    // r
    // }
    // _ => {
    // println!("creating a new room");
    // let new_room_id = room::create_new_room(&room_meta).await.unwrap();
    // room::assign_user_to_room(username.parse::<u32>().unwrap(), new_room_id, &room_meta).await;
    // 1u32
    // }
    // };

    // Register our peer with state which internally sets up some channels.
    let peer = Peer::new(state.clone(), lines, 1u32).await?;

    // A client has connected, let's let everyone know.
    {
        let mut state = state.lock().await;
        let msg = format!("{} has joined the chat", username);
        println!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    process_messages(peer, state.clone(), &username, addr).await?;

    Ok(())
}

/// Once a peer is connected process the messages it sends and receives
async fn process_messages(
    mut peer: Peer<'_>,
    state: Arc<Mutex<Shared>>,
    username: &str,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            // A message was received from a peer. Send it to the current user.
            Some(msg) = peer.rx.recv() => {
                // TODO augment the message with the users room id
                if msg.contains("room_id: 1") {
                    println!("sending message: {msg}");
                    peer.lines.send(&msg).await?;
                }

            }
            result = peer.lines.next() => match result {
                // A message was received from the current user, we should
                // broadcast this message to the other users.
                Some(Ok(msg)) => {
                    let mut state = state.lock().await;
                    let msg = format!("room_id: {} user: {}: {}", peer.room_id, username, msg);

                    state.broadcast(addr, &msg).await;
                }
                // An error occurred.
                Some(Err(e)) => {
                    println!(
                        "an error occurred while processing messages for {}; error = {:?}",
                        username,
                        e
                    );
                }
                // The stream has been exhausted.
                None => break,
            },
        }
    }

    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);
        // remove the peer from the room
    }

    Ok(())
}
