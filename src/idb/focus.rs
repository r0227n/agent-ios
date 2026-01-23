use crate::helpers::{with_client, CommandResult};

pub async fn run(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.focus().await?;
        Ok(())
    })
    .await
}
