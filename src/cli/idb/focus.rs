use crate::companion::CompanionState;
use crate::grpc::IdbClient;
use crate::types::Address;

pub async fn run(udid: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    // 3. Call focus RPC
    client.focus().await?;

    // 4. Success (no output for focus command)
    Ok(())
}
