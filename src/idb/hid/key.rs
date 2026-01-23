use crate::helpers::{with_client, CommandResult};
use agent_mobile_platform_ios::hid::events;

pub async fn run(keycode: u64, duration: Option<f64>, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let events = events::key_to_events(keycode, duration);
        client.hid(events).await?;
        Ok(())
    })
    .await
}
