use crate::companion::CompanionState;
use crate::grpc::IdbClient;
use crate::types::Address;
use std::io::Write;

pub async fn run(
    dest_path: String,
    udid: Option<String>,
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

    // 3. Take screenshot
    let image_data = client.screenshot().await?;

    // 4. Write to file or stdout
    if dest_path == "-" {
        // Write to stdout (binary mode)
        std::io::stdout().write_all(&image_data)?;
        std::io::stdout().flush()?;
    } else {
        // Write to file (atomic write)
        std::fs::write(&dest_path, &image_data)?;
    }

    Ok(())
}
