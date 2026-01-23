//! Connect to a companion
//!
//! This command connects to an idb_companion running on a specific target.
//! The companion info is stored in /tmp/idb/state for future use.

use agent_mobile_core::Address;
use agent_mobile_platform_ios::companion::CompanionState;
use agent_mobile_platform_ios::grpc::IdbClient;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(host: Option<String>, port: Option<u16>, udid: Option<String>) -> CommandResult {
    // Determine connection method
    let client = if let (Some(host), Some(port)) = (host.as_ref(), port) {
        // TCP connection to remote companion
        IdbClient::connect_tcp(host, port).await?
    } else if let Some(udid) = udid.as_ref() {
        // Look up companion from state file
        let state = CompanionState::default();
        let companion = state
            .find_by_udid(udid)
            .ok_or_else(|| format!("No companion found for udid: {}", udid))?;

        let address = companion
            .address()
            .ok_or_else(|| format!("No address for companion: {}", udid))?;

        match &address {
            Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
            Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
        }
    } else {
        return Err("Either --host/--port or --udid must be specified".into());
    };

    // Verify connection by calling describe
    let mut client = client;
    let target = client.describe(false).await?;

    println!("Connected to: {}", target.udid);
    println!("  Name: {}", target.name);
    println!("  State: {:?}", target.state);
    println!("  Type: {:?}", target.target_type);

    // Store connection info in state file
    if let (Some(host), Some(port)) = (host, port) {
        eprintln!("Connected via TCP to {}:{}", host, port);
    }

    Ok(())
}
