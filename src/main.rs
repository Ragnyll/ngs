use clap::Parser;

use log::LevelFilter;
use ngs::connection;

use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
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

    /// Amount of logging to allow from the server
    #[clap(long, default_value = "info")]
    pub log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let level = LevelFilter::from_str(&cli.log_level)?;
    SimpleLogger::new().with_level(level).init().unwrap();

    // start server
    let addr = format!("{}:{}", cli.host, cli.port).to_string();

    let listener = TcpListener::bind(&addr).await?;

    log::info!("Started Server on {addr}");

    // TODO: perhaps move this to its own service
    let room_meta = Arc::new(Mutex::new(HashMap::new()));
    log::info!("Received room service connection");

    log::info!("Listening for new connections");
    let state = Arc::new(Mutex::new(connection::Shared::new()));

    loop {
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let state = Arc::clone(&state);
        let room_meta = Arc::clone(&room_meta);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            log::info!("Accepted");
            if let Err(e) = connection::process(room_meta, state, stream, addr).await {
                log::error!("an error occurred; error = {:?}", e);
            }
        });
    }
}
