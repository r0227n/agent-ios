//! AI-optimized text input command.

use crate::cli::helpers::{with_client, CommandResult};
use crate::cli::idb::hid::events::text_to_events;

/// Type text on the device.
pub async fn run(text: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let events = text_to_events(&text)?;
        client.hid(events).await?;
        Ok(())
    })
    .await
}
