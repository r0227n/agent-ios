use crate::companion::state::CompanionState;
use crate::grpc::client::IdbClient;
use crate::grpc::idb::file_container::Kind as FileContainerKind;
use crate::grpc::idb::FileContainer;

/// Remove files or directories inside a container
pub async fn run(
    paths: Vec<String>,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read companion state
    let state = CompanionState::default();
    let companions = state.get_companions();

    if companions.is_empty() {
        return Err("No companions available. Make sure idb_companion is running.".into());
    }

    // Find target companion
    let companion = if let Some(ref udid_str) = udid {
        companions
            .iter()
            .find(|c| c.udid == *udid_str)
            .ok_or_else(|| format!("No companion found for UDID: {}", udid_str))?
    } else {
        // If no UDID specified, use the first available companion
        &companions[0]
    };

    // Get companion address
    let address = companion
        .address()
        .ok_or("Companion has no valid address")?;

    // Connect to companion
    let mut client = match &address {
        crate::types::Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
        crate::types::Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
    };

    // Build FileContainer
    let container = if let Some(bundle_id) = bundle_id {
        Some(FileContainer {
            kind: FileContainerKind::Application as i32,
            bundle_id,
        })
    } else {
        // Default to ROOT container
        Some(FileContainer {
            kind: FileContainerKind::Root as i32,
            bundle_id: String::new(),
        })
    };

    // Call rm
    client.rm(paths, container).await?;

    Ok(())
}
