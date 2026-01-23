//! Start the debug server

use crate::helpers::{with_client, CommandResult};

pub async fn run(bundle_id: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let commands = client.debugserver_start(bundle_id).await?;

        // Print LLDB bootstrap commands
        for cmd in commands {
            println!("{}", cmd);
        }

        Ok(())
    })
    .await
}
