use agent_mobile_core::{Compression, InstalledArtifact};

use crate::cli::helpers::{with_client, CommandResult, OutputFormat};
use serde_json::json;

pub async fn run(
    bundle_path: String,
    udid: Option<String>,
    make_debuggable: bool,
    override_mtime: bool,
    compression: Option<String>,
    format: OutputFormat,
) -> CommandResult {
    let json_output = format.is_json();
    // 1. Parse compression option
    let compression = compression.map(|s| s.parse::<Compression>()).transpose()?;

    with_client(udid.as_deref(), |mut client| async move {
        // 2. Call install RPC
        let mut response_stream = client
            .install(&bundle_path, make_debuggable, override_mtime, compression)
            .await?;

        // 3. Process streaming responses
        let mut artifact: Option<InstalledArtifact> = None;

        while let Some(response) = response_stream.message().await? {
            // Track progress
            let progress = if response.progress > 0.0 {
                Some(response.progress)
            } else {
                None
            };

            // Log progress (if not final response)
            if let Some(p) = progress {
                if p < 1.0 {
                    eprintln!("Progress: {:.0}%", p * 100.0);
                }
            }

            // Update artifact info
            if !response.name.is_empty() {
                artifact = Some(InstalledArtifact {
                    name: response.name,
                    uuid: if response.uuid.is_empty() {
                        None
                    } else {
                        Some(response.uuid)
                    },
                    progress,
                });
            }
        }

        // 5. Output result
        if let Some(art) = artifact {
            if json_output {
                let output = json!({
                    "installedAppBundleId": art.name,
                    "uuid": art.uuid,
                });
                println!("{}", serde_json::to_string(&output)?);
            } else {
                println!("Installed: {} {}", art.name, art.uuid.unwrap_or_default());
            }
        } else {
            return Err("No install response received".into());
        }

        Ok(())
    })
    .await
}
