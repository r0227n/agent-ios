//! focus command - Bring simulator window to foreground.
//!
//! Focuses the simulator window for the target device.

use crate::helpers::client::{with_client, CommandResult};

/// Execute the focus command.
pub async fn run(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.focus().await?;
        Ok(())
    })
    .await
}
