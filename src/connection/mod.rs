mod game_join;
mod peer;
pub mod shared;

use crate::connection::{peer::Peer, shared::Shared, game_join::GameJoinRequest};
use futures::SinkExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LinesCodec};

/// Process an individual chat client
pub async fn process_connection(
    state: Arc<Mutex<Shared<'_>>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    log::info!("Received new connection request, awaiting connection info");

    // accetps utf-8 encoded messages split by '\n'
    let game_join_request: GameJoinRequest = match lines.next().await {
        Some(Ok(line)) => serde_json::from_str(&line)?,
        _ => {
            println!("Invalid game join request from {addr}. Client disconnected.");
            return Ok(());
        }
    };
    // user acepted send them a waiting response
    let username = game_join_request.user_id;
    log::info!("username: {username:?} is waiting");

    // Register our peer with state which internally sets up some channels.
    let peer = Peer::new(state.clone(), lines, &username).await?;
    // the client has been processed succesfully, tell them they are waiting
    peer.tx
        .send(serde_json::to_string(&game_join::respond_wait())?)?;

    // the peer has been entered into the room, now just handle them
    process_messages(peer, state.clone(), &username).await?;

    Ok(())
}

/// Once a peer is connected process the messages it sends and receives
async fn process_messages(
    mut peer: Peer<'_>,
    state: Arc<Mutex<Shared<'_>>>,
    username: &str,
) -> Result<(), Box<dyn Error>> {
    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            // A message was received from a peer. Send it to the current user.
            Some(msg) = peer.rx.recv() => {
                println!("sending message: {msg}");
                peer.lines.send(&msg).await?;

            }
            result = peer.lines.next() => match result {
                // A message was received from the current user, we should
                // broadcast this message to the other users.
                Some(Ok(msg)) => {
                    let mut state = state.lock().await;
                    let msg = format!("user: {}: {}", username, msg);

                    state.broadcast(peer.user_id, &msg).await;
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

    // remove the peer if it is not streaming any messages
    {
        let mut state = state.lock().await;
        state.better_peers.remove(peer.user_id);
    }

    Ok(())
}
