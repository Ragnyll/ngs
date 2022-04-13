use clap::Parser;

use ngs::connection;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

/// NGS is a server for turn based games played by 1..n players over a tcp connection by passing
/// around deltas.
#[derive(Parser)]
pub struct Cli {
    /// The hostname to run the server off of
    #[clap(long, short, default_value = "127.0.0.1")]
    pub host: String,

    /// The port to run the server off of
    #[clap(long, short, default_value = "6142")]
    pub port: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // start server
    let addr = format!("{}:{}", cli.host, cli.port).to_string();

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

