//! dsym install command implementation

use crate::cli::helpers::{with_client, CommandResult, OutputFormat};
use crate::types::Compression;
use serde_json::json;

/// Install dSYM symbols to the target
pub async fn install(
    dsym_path: String,
    bundle_id: Option<String>,
    compression: Option<String>,
    format: OutputFormat,
    udid: Option<String>,
) -> CommandResult {
    let json_output = format.is_json();
    // Parse compression option
    let compression = compression.map(|s| s.parse::<Compression>()).transpose()?;

    with_client(udid.as_deref(), |mut client| async move {
        // Call install_dsym RPC
        let mut response_stream = client
            .install_dsym(&dsym_path, bundle_id, compression)
            .await?;

        // Process streaming responses
        let mut artifact_name = String::new();

        while let Some(response) = response_stream.message().await? {
            // Log progress (if not final response)
            if response.progress > 0.0 && response.progress < 1.0 && !json_output {
                eprintln!("Progress: {:.0}%", response.progress * 100.0);
            }

            // Update artifact info
            if !response.name.is_empty() {
                artifact_name = response.name;
            }
        }

        // Output result
        if !artifact_name.is_empty() {
            if json_output {
                println!("{}", json!({"dsym": artifact_name}));
            } else {
                println!("Installed: {}", artifact_name);
            }
        } else {
            return Err("No install response received".into());
        }

        Ok(())
    })
    .await
}
