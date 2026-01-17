use crate::cli::helpers::{with_client, CommandResult};
use crate::cli::idb::hid::events;

pub async fn run(x: f64, y: f64, duration: Option<f64>, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let events = events::tap_to_events(x, y, duration);
        client.hid(events).await?;
        Ok(())
    })
    .await
}
