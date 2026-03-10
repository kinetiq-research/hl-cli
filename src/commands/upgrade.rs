use crate::error::CliError;

/// Try to find the source directory by checking where the current binary was installed from.
/// Falls back to well-known local paths, then to git.
fn find_source_dir() -> Option<std::path::PathBuf> {
    // Check common local paths
    let candidates = [
        // Relative to HOME
        dirs_home().map(|h| h.join("workspace/trading/hl-cli")),
        dirs_home().map(|h| h.join("workspace/hl-cli")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.join("Cargo.toml").exists() {
            return Some(candidate);
        }
    }
    None
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}

pub fn run() -> Result<(), CliError> {
    // First try local source
    if let Some(source_dir) = find_source_dir() {
        println!("Upgrading hl from local source: {}", source_dir.display());

        // Pull latest if it's a git repo
        if source_dir.join(".git").exists() {
            println!("Pulling latest changes...");
            let _ = std::process::Command::new("git")
                .args(["pull"])
                .current_dir(&source_dir)
                .status();
        }

        let status = std::process::Command::new("cargo")
            .args(["install", "--path", ".", "--force"])
            .current_dir(&source_dir)
            .status();

        return match status {
            Ok(s) if s.success() => {
                println!("Upgrade complete!");
                Ok(())
            }
            Ok(s) => Err(CliError::Api(format!(
                "cargo install failed with exit code: {}",
                s.code().unwrap_or(-1)
            ))),
            Err(e) => Err(CliError::Api(format!(
                "Failed to run cargo install: {e}"
            ))),
        };
    }

    // Fallback: install from git
    println!("Upgrading hl from git...");
    let status = std::process::Command::new("cargo")
        .args([
            "install",
            "--git",
            "https://github.com/kinetiq-research/hl-cli",
            "--force",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Upgrade complete!");
            Ok(())
        }
        Ok(s) => Err(CliError::Api(format!(
            "cargo install failed with exit code: {}",
            s.code().unwrap_or(-1)
        ))),
        Err(e) => Err(CliError::Api(format!(
            "Failed to run cargo install: {e}. Make sure cargo is installed."
        ))),
    }
}
