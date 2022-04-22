use log::LevelFilter;
use ngs::connection;
use ngs::cli::Cli;

use simple_logger::SimpleLogger;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse_cli();

    let level = LevelFilter::from_str(&cli.log_level)?;
    SimpleLogger::new().with_level(level).init().unwrap();

    // start server
    let addr = format!("{}:{}", cli.host, cli.port).to_string();

    let listener = TcpListener::bind(&addr).await?;

    log::info!("Started Server on {addr}");

    // TODO: perhaps move this to its own service
    log::info!("Received room service connection");

    log::info!("Listening for new connections");
    let state = Arc::new(Mutex::new(connection::shared::Shared::new()));

    loop {
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let state = Arc::clone(&state);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            log::info!("Accepted");
            if let Err(e) = connection::process(state, stream, addr).await {
                log::error!("an error occurred; error = {:?}", e);
            }
        });
    }
}
