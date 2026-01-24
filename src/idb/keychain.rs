//! keychain command - Keychain operations.
//!
//! Clear the device's keychain.

use crate::helpers::client::{with_client, CommandResult};

/// Clear all keychain items.
pub async fn clear(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.clear_keychain().await?;
        Ok(())
    })
    .await
}
