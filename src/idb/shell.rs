//! Interactive shell command implementation
//!
//! Provides a REPL for executing multiple IDB commands with a single connection

use std::io::{self, BufRead, Write};

use crate::helpers::CommandResult;

/// Run an interactive shell for chaining multiple IDB commands
pub async fn run(no_prompt: bool, _udid: Option<String>) -> CommandResult {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let prompt = if no_prompt { "" } else { "idb> " };

    eprintln!("Interactive IDB shell. Type 'exit' to quit.");
    eprintln!("Note: Each command creates a new connection to the companion.");

    loop {
        // Flush output streams
        stdout.flush()?;
        io::stderr().flush()?;

        // Print prompt
        if !no_prompt {
            print!("{}", prompt);
            stdout.flush()?;
        }

        // Read line
        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;

        // EOF
        if bytes_read == 0 {
            break;
        }

        // Trim whitespace
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Handle exit command
        if line == "exit" || line == "quit" {
            break;
        }

        // Handle help command
        if line == "help" {
            println!("Available commands:");
            println!("  exit, quit     - Exit the shell");
            println!("  help           - Show this help");
            println!("  <idb command>  - Run any idb subcommand");
            println!();
            println!("Example: list-targets --human");
            println!("SUCCESS=1");
            continue;
        }

        // Parse the command line using shlex
        let args = match shlex::split(line) {
            Some(args) => args,
            None => {
                eprintln!("Error: Invalid command syntax (unbalanced quotes)");
                println!("SUCCESS=0");
                continue;
            }
        };

        if args.is_empty() {
            continue;
        }

        // Execute the command by spawning a subprocess
        // This is the simplest approach that avoids complex refactoring
        // while maintaining compatibility with Python idb shell behavior
        let result = execute_command(&args).await;

        match result {
            Ok(_) => {
                println!("SUCCESS=1");
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                println!("SUCCESS=0");
            }
        }
    }

    Ok(())
}

/// Execute a command by spawning a subprocess
async fn execute_command(args: &[String]) -> CommandResult {
    use std::process::Command;

    // Get the path to the current executable
    let exe = std::env::current_exe()?;

    // Build command: agent-mobile idb <args>
    let mut cmd = Command::new(&exe);
    cmd.arg("idb");
    cmd.args(args);

    // Execute and wait
    let output = cmd.output()?;

    // Print stdout
    if !output.stdout.is_empty() {
        io::stdout().write_all(&output.stdout)?;
    }

    // Print stderr
    if !output.stderr.is_empty() {
        io::stderr().write_all(&output.stderr)?;
    }

    // Check exit status
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Command exited with status: {}",
            output.status.code().unwrap_or(-1)
        )
        .into())
    }
}
