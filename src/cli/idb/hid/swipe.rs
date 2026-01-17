use crate::cli::helpers::{with_client, CommandResult};
use crate::cli::idb::hid::events;

pub async fn run(
    x_start: f64,
    y_start: f64,
    x_end: f64,
    y_end: f64,
    duration: Option<f64>,
    delta: Option<f64>,
    udid: Option<String>,
) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let events = events::swipe_to_events((x_start, y_start), (x_end, y_end), duration, delta);
        client.hid(events).await?;
        Ok(())
    })
    .await
}
