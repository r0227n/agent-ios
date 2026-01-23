use crate::helpers::{with_client, CommandResult};

pub async fn clear(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.clear_keychain().await?;
        Ok(())
    })
    .await
}
