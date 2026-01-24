//! memory command - Memory simulation.
//!
//! Simulate memory warnings on the device.

use crate::helpers::client::{with_client, CommandResult};

/// Simulate a memory warning.
pub async fn simulate_warning(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.simulate_memory_warning().await?;
        Ok(())
    })
    .await
}
