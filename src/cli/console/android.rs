//! Android console streaming implementation
//!
//! Streams Android device logcat using adb command.
//! Press Ctrl+C to stop streaming.

use crate::cli::helpers::{setup_ctrl_c_handler, CommandResult, OutputWriter};
use std::io::Write;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Stream Android console output via adb logcat
///
/// Spawns an `adb logcat` process and streams its output in real-time.
/// The stream continues until Ctrl+C is pressed.
pub async fn run(udid: Option<String>, mut writer: OutputWriter) -> CommandResult {
    // Setup stop signal for Ctrl+C
    let mut stop_rx = setup_ctrl_c_handler();

    // Build adb logcat command
    let mut cmd = Command::new("adb");
    if let Some(s) = &udid {
        cmd.args(["-s", s]);
    }
    cmd.arg("logcat");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Spawn the process
    let mut child = cmd.spawn().map_err(|e| {
        format!(
            "Failed to start adb logcat: {}. Make sure adb is installed and in PATH.",
            e
        )
    })?;

    let stdout = child.stdout.take().ok_or("Failed to capture adb stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    // Process stream with graceful shutdown
    loop {
        tokio::select! {
            // Check for stop signal (Ctrl+C)
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() {
                    // Kill the child process gracefully
                    child.kill().await.ok();
                    break;
                }
            }

            // Read lines from logcat
            line = reader.next_line() => {
                match line? {
                    Some(line) => {
                        writeln!(writer, "{}", line)?;
                        writer.flush()?;
                    }
                    None => break, // Stream ended
                }
            }
        }
    }

    // Final newline for clean exit
    writeln!(writer)?;

    Ok(())
}
