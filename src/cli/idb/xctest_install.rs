use crate::cli::helpers::{with_client, CommandResult};
use crate::types::{Compression, InstalledArtifact};
use serde_json::json;

pub async fn run(
    test_bundle_path: String,
    udid: Option<String>,
    skip_signing: bool,
    compression: Option<String>,
    json_output: bool,
) -> CommandResult {
    // Parse compression option
    let compression = compression.map(|s| s.parse::<Compression>()).transpose()?;

    with_client(udid.as_deref(), |mut client| async move {
        // Call install_xctest RPC
        let mut response_stream = client
            .install_xctest(&test_bundle_path, skip_signing, compression)
            .await?;

        // Process streaming responses
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
                if p < 1.0 && !json_output {
                    eprintln!("Installed {:.0}%", p * 100.0);
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

        // Output result
        if let Some(art) = artifact {
            if json_output {
                let output = json!({
                    "installedTestBundleId": art.name,
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
