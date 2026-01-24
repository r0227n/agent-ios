//! photos command - Photo library operations.
//!
//! Clear all photos from the device.

use crate::helpers::client::{with_client, CommandResult};

/// Clear all photos from the device.
pub async fn clear(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.photos_clear().await?;
        Ok(())
    })
    .await
}
