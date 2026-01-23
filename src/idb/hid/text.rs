use crate::helpers::{with_client, CommandResult};
use crate::idb::hid::events;

pub async fn run(text: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let events = events::text_to_events(&text)?;
        client.hid(events).await?;
        Ok(())
    })
    .await
}
