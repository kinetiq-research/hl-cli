use std::io::{self, BufRead, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::cli::Network;
use crate::error::CliError;

fn env_path() -> Result<PathBuf, CliError> {
    let home = std::env::var("HOME")
        .map_err(|_| CliError::Api("HOME environment variable not set".into()))?;
    Ok(PathBuf::from(home).join(".hl.env"))
}

fn prompt(label: &str, hint: &str) -> String {
    eprint!("{label}");
    if !hint.is_empty() {
        eprint!(" ({hint})");
    }
    eprint!(": ");
    io::stderr().flush().ok();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).ok();
    line.trim().to_string()
}

pub fn run(
    private_key: Option<String>,
    address: Option<String>,
    network: Option<Network>,
    force: bool,
    no_skill: bool,
) -> Result<(), CliError> {
    let path = env_path()?;

    if path.exists() && !force {
        eprintln!("Config already exists: {}", path.display());
        eprintln!("Use --force to overwrite.");
        return Ok(());
    }

    // Use flags if provided, otherwise prompt interactively
    let key = private_key.unwrap_or_else(|| {
        prompt(
            "API Wallet private key",
            "from Hyperliquid UI → Settings → API Wallets, or leave blank",
        )
    });

    let addr = address.unwrap_or_else(|| {
        prompt("Wallet address", "0x..., for read-only operations")
    });

    let net = match network {
        Some(Network::Testnet) => "testnet".to_string(),
        Some(Network::Mainnet) => "mainnet".to_string(),
        None => {
            let input = prompt("Default network", "mainnet/testnet, default: mainnet");
            if input.is_empty() {
                "mainnet".to_string()
            } else {
                input
            }
        }
    };

    let mut contents = String::new();
    if !key.is_empty() {
        contents.push_str(&format!("HL_PRIVATE_KEY={key}\n"));
    }
    if !addr.is_empty() {
        contents.push_str(&format!("HL_ADDRESS={addr}\n"));
    }
    contents.push_str(&format!("HL_NETWORK={net}\n"));

    std::fs::write(&path, &contents)
        .map_err(|e| CliError::Api(format!("Failed to write {}: {e}", path.display())))?;

    // Restrict file permissions to owner-only (0600) since it may contain private keys
    #[cfg(unix)]
    {
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)
            .map_err(|e| CliError::Api(format!("Failed to set permissions: {e}")))?;
    }

    eprintln!("Config written to {}", path.display());

    // Install the Claude Code skill unless opted out
    if !no_skill {
        crate::commands::install_skill::run(false, force)?;
    }

    Ok(())
}
