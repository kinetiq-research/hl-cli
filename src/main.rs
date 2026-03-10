use clap::Parser;

mod cli;
mod client;
mod commands;
mod error;
mod output;

use cli::Cli;
use error::CliError;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), CliError> {
    commands::dispatch(cli).await
}
