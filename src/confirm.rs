use std::io::{self, Write};

/// Prompt the user for confirmation before a write operation.
/// Returns Ok(()) if confirmed, Err if denied.
/// If `skip` is true, skips the prompt (--yes flag).
pub fn confirm_action(description: &str, skip: bool) -> Result<(), crate::error::CliError> {
    if skip {
        return Ok(());
    }

    eprint!("\x1b[1;33m⚠ {description}\x1b[0m\n  Proceed? [y/N] ");
    io::stderr().flush().ok();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| crate::error::CliError::Api(format!("Failed to read input: {e}")))?;

    let answer = input.trim().to_lowercase();
    if answer == "y" || answer == "yes" {
        Ok(())
    } else {
        Err(crate::error::CliError::Api("Cancelled by user".into()))
    }
}
