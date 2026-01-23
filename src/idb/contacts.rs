use crate::helpers::{with_client, CommandResult};

pub async fn update(db_path: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.contacts_update(&db_path).await?;
        Ok(())
    })
    .await
}

pub async fn clear(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.contacts_clear().await?;
        Ok(())
    })
    .await
}
