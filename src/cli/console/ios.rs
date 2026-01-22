//! iOS console streaming implementation
//!
//! Streams iOS device/simulator logs using idb gRPC log API.
//! Press Ctrl+C to stop streaming.

use crate::cli::helpers::{setup_ctrl_c_handler, with_client, CommandResult};
use crate::grpc::idb::log_request::Source as LogSource;
use std::io::Write;

/// Stream iOS console output
///
/// Connects to idb_companion and streams device logs in real-time.
/// The stream continues until Ctrl+C is pressed.
pub async fn run(udid: Option<String>) -> CommandResult {
    // Setup stop signal for Ctrl+C
    let mut stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        // Start streaming logs from target device
        let mut response_stream = client.log(LogSource::Target, vec![]).await?;

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

        // Final newline for clean exit
        println!();

        Ok(())
    })
    .await
}
