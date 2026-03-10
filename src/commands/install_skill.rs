use std::path::PathBuf;

use crate::error::CliError;

// Embed the skill file at compile time — works even after `cargo install`
const SKILL_CONTENT: &str = include_str!("../../.claude/commands/hl.md");

fn write_skill(path: &PathBuf, force: bool) -> Result<(), CliError> {
    if path.exists() && !force {
        eprintln!("Skill already exists: {}", path.display());
        eprintln!("Use --force to overwrite.");
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CliError::Api(format!("Failed to create directory: {e}")))?;
    }

    std::fs::write(path, SKILL_CONTENT)
        .map_err(|e| CliError::Api(format!("Failed to write {}: {e}", path.display())))?;

    eprintln!("Skill installed: {}", path.display());
    Ok(())
}

pub fn run(project: bool, force: bool) -> Result<(), CliError> {
    // Always install to ~/.claude/commands/hl.md (user-global)
    let home = std::env::var("HOME")
        .map_err(|_| CliError::Api("HOME environment variable not set".into()))?;
    let global_path = PathBuf::from(&home).join(".claude/commands/hl.md");
    write_skill(&global_path, force)?;

    // Optionally also install into the current project
    if project {
        let project_path = PathBuf::from(".claude/commands/hl.md");
        write_skill(&project_path, force)?;
    }

    eprintln!("\nUsage: invoke with /hl in any Claude Code session.");
    Ok(())
}
