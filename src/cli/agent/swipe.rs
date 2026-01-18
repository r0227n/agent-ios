//! AI-optimized swipe command.

use crate::cli::helpers::{with_client, CommandResult};
use crate::cli::idb::hid::events::swipe_to_events;

/// Swipe from start to end coordinates.
pub async fn run(
    x_start: f64,
    y_start: f64,
    x_end: f64,
    y_end: f64,
    duration: f64,
    udid: Option<String>,
) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client
            .hid(swipe_to_events(
                (x_start, y_start),
                (x_end, y_end),
                Some(duration),
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}
