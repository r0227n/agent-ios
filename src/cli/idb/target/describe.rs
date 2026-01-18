//! Describe a target

use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(udid: Option<String>, diagnostics: bool) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let target = client.describe(diagnostics).await?;

        // Output in JSON format
        println!("{}", serde_json::to_string_pretty(&target)?);

        Ok(())
    })
    .await
}
