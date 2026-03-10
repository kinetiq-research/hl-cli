use crate::cli::{Cli, Command};
use crate::error::CliError;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub async fn run() -> Result<(), CliError> {
    println!("Hyperliquid interactive shell. Type commands without 'hl' prefix.");
    println!("Examples: state, positions, book ETH, order place ETH buy --size 0.1 --price 3000");
    println!("Type 'exit' or 'quit' to leave, Ctrl+C to cancel.\n");

    let mut rl = DefaultEditor::new()
        .map_err(|e| CliError::Api(format!("Failed to init readline: {e}")))?;

    let history_path = dirs_hint();
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    loop {
        let readline = rl.readline("hl> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }

                let _ = rl.add_history_entry(line);

                // Parse the line as if it were CLI args
                let mut args = vec!["hl".to_string()];
                args.extend(shell_words(line));

                match Cli::try_parse_from(&args) {
                    Ok(cli) => {
                        // Don't allow nested shell
                        if matches!(cli.command, Command::Shell) {
                            println!("Already in shell mode");
                            continue;
                        }
                        if let Err(e) = Box::pin(crate::commands::dispatch(cli)).await {
                            eprintln!("Error: {e}");
                        }
                    }
                    Err(e) => {
                        // clap prints help/error
                        let _ = e.print();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                eprintln!("Readline error: {e}");
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    println!("Bye!");
    Ok(())
}

fn dirs_hint() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".hl_history"))
}

/// Simple shell word splitting (handles quoted strings).
fn shell_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }
        if ch == '\\' && !in_single_quote {
            escape_next = true;
            continue;
        }
        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            continue;
        }
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            continue;
        }
        if ch.is_whitespace() && !in_single_quote && !in_double_quote {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}
