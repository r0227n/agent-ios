use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(url: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.open_url(&url).await?;
        Ok(())
    })
    .await
}
