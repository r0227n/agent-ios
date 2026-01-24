//! url command - Open URLs on the device.
//!
//! Open a URL in the device's default browser or registered app.

use crate::helpers::client::{with_client, CommandResult};

/// Execute the URL open command.
pub async fn run(url: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.open_url(&url).await?;
        Ok(())
    })
    .await
}
