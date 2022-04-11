use ngs::connection;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // start server
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    let listener = TcpListener::bind(&addr).await?;

    println!("server running on {}", addr);
    // start room service
    let room_meta = Arc::new(Mutex::new(HashMap::new()));

    // listen for and process connections
    let state = Arc::new(Mutex::new(connection::Shared::new()));

    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let state = Arc::clone(&state);
        let room_meta = Arc::clone(&room_meta);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            println!("accepted connection");
            if let Err(e) = connection::process(room_meta, state, stream, addr).await {
                println!("an error occurred; error = {:?}", e);
            }
        });
    }
}

