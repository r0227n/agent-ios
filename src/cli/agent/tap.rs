//! AI-optimized tap command.

use crate::cli::helpers::{with_client, CommandResult};
use crate::cli::idb::hid::events::tap_to_events;

/// Tap at the specified coordinates.
pub async fn run(x: f64, y: f64, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.hid(tap_to_events(x, y, None)).await?;
        Ok(())
    })
    .await
}
