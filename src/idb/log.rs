//! log command - Stream device logs.
//!
//! Obtains and streams logs from the target device or companion.

use agent_mobile_platform_ios::proto::idb::log_request::Source as LogSource;

use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::signal::setup_ctrl_c_handler;
use std::io::Write;

/// Execute the log command.
pub async fn run(
    udid: Option<String>,
    source: String,
    log_arguments: Vec<String>,
) -> CommandResult {
    // Parse source option
    let log_source = match source.as_str() {
        "companion" => LogSource::Companion,
        _ => LogSource::Target,
    };

    // Normalize log arguments (remove leading "--" if present)
    let arguments = normalize_log_arguments(log_arguments);

    // Setup stop signal for Ctrl+C
    let mut stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        // Start streaming logs
        let mut response_stream = client.log(log_source, arguments).await?;

        // Process stream with graceful shutdown
        loop {
            tokio::select! {
                // Check for stop signal (Ctrl+C)
                _ = stop_rx.changed() => {
                    if *stop_rx.borrow() {
                        break;
                    }
                }

                // Process response stream
                response = response_stream.message() => {
                    match response? {
                        Some(log_response) => {
                            // Write log output to stdout
                            let output = &log_response.output;
                            std::io::stdout().write_all(output)?;
                            std::io::stdout().flush()?;
                        }
                        None => break, // Stream ended
                    }
                }
            }
        }

        // Final newline (matching Python behavior)
        println!();

        Ok(())
    })
    .await
}

/// Normalize log arguments by removing leading "--" separator
fn normalize_log_arguments(mut args: Vec<String>) -> Vec<String> {
    if !args.is_empty() && args[0] == "--" {
        args.remove(0);
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_log_arguments_with_separator() {
        let args = vec!["--".to_string(), "--style".to_string(), "json".to_string()];
        let normalized = normalize_log_arguments(args);
        assert_eq!(normalized, vec!["--style", "json"]);
    }

    #[test]
    fn test_normalize_log_arguments_without_separator() {
        let args = vec!["--style".to_string(), "json".to_string()];
        let normalized = normalize_log_arguments(args);
        assert_eq!(normalized, vec!["--style", "json"]);
    }

    #[test]
    fn test_normalize_log_arguments_empty() {
        let args: Vec<String> = vec![];
        let normalized = normalize_log_arguments(args);
        assert!(normalized.is_empty());
    }
}
