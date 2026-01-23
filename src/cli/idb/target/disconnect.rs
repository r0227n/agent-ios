//! Disconnect from a companion
//!
//! This command disconnects from an idb_companion for a specific target.
//! Note: In the current architecture, disconnection simply means stopping
//! communication - the companion process continues running.

use agent_mobile_platform_ios::companion::CompanionState;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(udid: String) -> CommandResult {
    // Verify the companion exists in state
    let state = CompanionState::default();
    let companion = state
        .find_by_udid(&udid)
        .ok_or_else(|| format!("No companion found for udid: {}", udid))?;

    // In the Python idb, disconnect just removes the companion from the local state
    // The actual companion process keeps running
    // Since we're stateless (we look up companions on each command), we just verify existence

    let address = companion.address();
    println!("Disconnected from: {}", udid);
    if let Some(addr) = address {
        println!("  Companion was at: {:?}", addr);
    }

    Ok(())
}
