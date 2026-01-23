use crate::helpers::{with_client, CommandResult};

pub async fn run(bundle_id: String, json_payload: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.send_notification(&bundle_id, &json_payload).await?;
        Ok(())
    })
    .await
}
