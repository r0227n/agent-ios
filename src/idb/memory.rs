use crate::helpers::{with_client, CommandResult};

pub async fn simulate_warning(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.simulate_memory_warning().await?;
        Ok(())
    })
    .await
}
