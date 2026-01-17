use crate::cli::helpers::{with_client, CommandResult};
use crate::cli::idb::hid::events;

pub async fn run(key_sequence: Vec<u64>, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let events = events::key_sequence_to_events(key_sequence);
        client.hid(events).await?;
        Ok(())
    })
    .await
}
