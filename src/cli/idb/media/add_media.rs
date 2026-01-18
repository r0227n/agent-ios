use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(file_paths: Vec<String>, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.add_media(file_paths).await?;
        Ok(())
    })
    .await
}
