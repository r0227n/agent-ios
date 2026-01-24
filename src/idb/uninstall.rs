//! uninstall command - Uninstall an application from the device.
//!
//! Removes an installed app by its bundle identifier.

use crate::helpers::client::{with_client, CommandResult};

/// Execute the uninstall command.
pub async fn run(bundle_id: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.uninstall(&bundle_id).await?;
        Ok(())
    })
    .await
}
