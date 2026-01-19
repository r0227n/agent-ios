//! AI-optimized screenshot command.

use crate::cli::helpers::{with_client, CommandResult};

/// Take a screenshot and save to the specified path.
pub async fn run(path: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let image_data = client.screenshot().await?;
        std::fs::write(&path, image_data)?;
        println!("{}", path);
        Ok(())
    })
    .await
}
