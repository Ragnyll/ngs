use clap::Parser;

/// NGS is a server for turn based games played by 1..n players over a tcp connection by passing
/// around deltas.
#[derive(clap::Parser)]
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

impl Cli {
    // for some reason this needs a wrapper because the main crate cant find the derived trait.
    pub fn parse_cli() -> Cli {
        Cli::parse()
    }
}
