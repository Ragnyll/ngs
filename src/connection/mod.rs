mod game_join;
mod peer;
pub mod shared;

use crate::connection::{peer::Peer, shared::Shared};
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LinesCodec};

/// Process an individual chat client
pub async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    log::info!("Received new connection request, awaiting connection info");
    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            // TODO handle the connection message here
            // TODO timeout a connection if not received the connection string
            println!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(());
        }
    };

    // Register our peer with state which internally sets up some channels.
    let peer = Peer::new(state.clone(), lines, 1u32).await?;

    // A client has connected. Broadcast their arrival to the peers in the room
    {
        let mut state = state.lock().await;
        let msg = format!("{} has joined the game.", username);
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
