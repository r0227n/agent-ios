//! dylib install command implementation

use crate::cli::helpers::{with_client, CommandResult, OutputFormat};
use serde_json::json;

/// Install a dylib to the target
pub async fn install(
    dylib_path: String,
    output: OutputFormat,
    udid: Option<String>,
) -> CommandResult {
    let json_output = output.is_json();
    with_client(udid.as_deref(), |mut client| async move {
        // Call install_dylib RPC
        let mut response_stream = client.install_dylib(&dylib_path).await?;

        // Process streaming responses
        let mut artifact_name = String::new();
        let mut artifact_uuid = String::new();

        while let Some(response) = response_stream.message().await? {
            // Log progress (if not final response)
            if response.progress > 0.0 && response.progress < 1.0 && !json_output {
                eprintln!("Progress: {:.0}%", response.progress * 100.0);
            }

            // Update artifact info
            if !response.name.is_empty() {
                artifact_name = response.name;
            }
            if !response.uuid.is_empty() {
                artifact_uuid = response.uuid;
            }
        }

        // Output result
        if !artifact_name.is_empty() {
            if json_output {
                println!("{}", json!({"dylib": artifact_name, "uuid": artifact_uuid}));
            } else {
                println!("Installed: {} {}", artifact_name, artifact_uuid);
            }
        } else {
            return Err("No install response received".into());
        }

        Ok(())
    })
    .await
}
