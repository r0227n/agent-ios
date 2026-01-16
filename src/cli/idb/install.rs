use crate::companion::CompanionState;
use crate::grpc::IdbClient;
use crate::types::{Address, Compression, InstalledArtifact};
use serde_json::json;

pub async fn run(
    bundle_path: String,
    udid: Option<String>,
    make_debuggable: bool,
    override_mtime: bool,
    compression: Option<String>,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Parse compression option
    let compression = compression.map(|s| s.parse::<Compression>()).transpose()?;

    // 2. Resolve companion by UDID
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

    // 3. Connect to companion
    let address = companion
        .address()
        .ok_or("Companion has no valid address")?;

    let mut client = match &address {
        Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
        Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
    };

    // 4. Call install RPC
    let mut response_stream = client
        .install(&bundle_path, make_debuggable, override_mtime, compression)
        .await?;

    // 5. Process streaming responses
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

    // 6. Output result
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
}
