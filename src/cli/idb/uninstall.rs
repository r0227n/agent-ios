use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(bundle_id: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.uninstall(&bundle_id).await?;
        Ok(())
    })
    .await
}
