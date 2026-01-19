//! AI-optimized accessibility command.

use crate::cli::helpers::{with_client, CommandResult};

/// Get the accessibility tree for the current screen.
pub async fn run(nested: bool, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let info = client.accessibility_info(None, nested).await?;
        println!("{}", info);
        Ok(())
    })
    .await
}
