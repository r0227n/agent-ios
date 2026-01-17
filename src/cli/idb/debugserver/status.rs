//! Get the status of the debug server

use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let status = client.debugserver_status().await?;

        match status {
            Some(commands) => {
                // Print LLDB bootstrap commands
                for cmd in commands {
                    println!("{}", cmd);
                }
            }
            None => {
                println!("Not Running");
            }
        }

        Ok(())
    })
    .await
}
