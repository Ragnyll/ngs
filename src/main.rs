use ngs::connection;
use ngs::room;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;


type RoomState = Arc<Mutex<connection::Shared>>;
#[tokio::main]

async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    // Bind a TCP listener to the socket address.
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    println!("server running on {}", addr);

    // room_id to rom_state
    let rooms = Arc::new(Mutex::new(HashMap::<u32, RoomState>::new()));
    let room_meta: Arc<Mutex<room::RoomMeta>> = Arc::new(Mutex::new(HashMap::new()));
    let state = Arc::new(Mutex::new(connection::Shared::new()));

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let state = Arc::clone(&state);
        let rooms = Arc::clone(&rooms);
        let room_meta = Arc::clone(&room_meta);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            println!("accepted connection");
            if let Err(e) = process(room_meta, rooms, state, stream, addr).await {
                println!("an error occurred; error = {:?}", e);
            }
        });
    }
}


/// Process an individual chat client
async fn process(
    room_meta: Arc<Mutex<room::RoomMeta>>,
    rooms: Arc<Mutex<HashMap<u32, RoomState>>>,
    state: Arc<Mutex<connection::Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    lines.send("Please enter your username:").await?;
    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            println!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(());
        }
    };

    lines.send("Please enter the user_id of whom you want to pair with").await?;
    let requested_user_id: u32 = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            println!("Failed to get user_id_request from {}. Client disconnected.", addr);
            return Ok(());
        }
    }.parse().unwrap();

    // if requested user_id is found then get the room needed, otherwise create a new room
    match room::find_room_with_user(requested_user_id, room_meta).await {
        Some(r) => println!("found the user in room {r}"),
        _ => println!("creating a new room")
    };


    // Register our peer with state which internally sets up some channels.
    let mut peer = connection::Peer::new(state.clone(), lines).await?;

    // A client has connected, let's let everyone know.
    {
        let mut state = state.lock().await;
        let msg = format!("{} has joined the chat", username);
        println!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            // A message was received from a peer. Send it to the current user.
            Some(msg) = peer.rx.recv() => {
                peer.lines.send(&msg).await?;
            }
            result = peer.lines.next() => match result {
                // A message was received from the current user, we should
                // broadcast this message to the other users.
                Some(Ok(msg)) => {
                    let mut state = state.lock().await;
                    let msg = format!("{}: {}", username, msg);

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
    }

    Ok(())
}
