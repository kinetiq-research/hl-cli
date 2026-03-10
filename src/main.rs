use clap::Parser;

mod cli;
mod client;
mod commands;
pub mod confirm;
mod error;
mod output;

use cli::Cli;
use error::CliError;

#[tokio::main]
async fn main() {
    // Load .env: try ~/.hl.env first (global), then .env in CWD
    if let Some(home) = std::env::var_os("HOME") {
        let global_env = std::path::PathBuf::from(home).join(".hl.env");
        let _ = dotenvy::from_path(&global_env);
    }
    let _ = dotenvy::dotenv(); // CWD .env overrides

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), CliError> {
    // Watch mode: re-run the command every N seconds
    if let Some(interval) = cli.watch {
        if interval == 0 {
            return Err(CliError::InvalidArg("Watch interval must be > 0".into()));
        }
        loop {
            // Clear screen
            print!("\x1b[2J\x1b[H");
            // Re-parse to get a fresh Cli (we can't clone easily, so re-dispatch)
            let fresh_cli = Cli::parse();
            if let Err(e) = commands::dispatch_once(&fresh_cli).await {
                eprintln!("Error: {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    } else {
        commands::dispatch(cli).await
    }
}
