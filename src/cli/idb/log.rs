use crate::companion::CompanionState;
use crate::grpc::idb::log_request::Source as LogSource;
use crate::grpc::IdbClient;
use crate::types::Address;
use std::io::Write;
use tokio::sync::watch;

pub async fn run(
    udid: Option<String>,
    source: String,
    log_arguments: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Resolve companion by UDID
    let state = CompanionState::default();
    let companion = match udid.as_deref() {
        Some(u) => state
            .find_by_udid(u)
            .ok_or_else(|| format!("No companion found for UDID: {}", u))?,
        None => state
            .get_companions()
            .into_iter()
            .next()
            .ok_or("No companions available. Run 'idb_companion' first.")?,
    };

    // 2. Connect to companion
    let address = companion
        .address()
        .ok_or("Companion has no valid address")?;

    let mut client = match &address {
        Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
        Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
    };

    // 3. Parse source option
    let log_source = match source.as_str() {
        "companion" => LogSource::Companion,
        _ => LogSource::Target,
    };

    // 4. Normalize log arguments (remove leading "--" if present)
    let arguments = normalize_log_arguments(log_arguments);

    // 5. Setup stop signal for Ctrl+C
    let (stop_tx, mut stop_rx) = watch::channel(false);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        stop_tx.send(true).ok();
    });

    // 6. Start streaming logs
    let mut response_stream = client.log(log_source, arguments).await?;

    // 7. Process stream with graceful shutdown
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
