//! Stop the debug server

use crate::helpers::client::{with_client, CommandResult};

pub async fn run(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.debugserver_stop().await?;
        Ok(())
    })
    .await
}
