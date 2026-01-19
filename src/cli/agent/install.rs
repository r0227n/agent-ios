//! AI-optimized app install command.

use crate::cli::helpers::{with_client, CommandResult};
use std::path::Path;

/// Install an application from a bundle path.
pub async fn run(bundle_path: String, udid: Option<String>) -> CommandResult {
    let path = Path::new(&bundle_path);

    if !path.exists() {
        return Err(format!("Bundle path does not exist: {}", bundle_path).into());
    }

    with_client(udid.as_deref(), |mut client| async move {
        let mut stream = client.install(&bundle_path, false, false, None).await?;

        // Process the streaming response to get the final result
        use tokio_stream::StreamExt;
        while let Some(response) = stream.next().await {
            if let Ok(resp) = response {
                if !resp.name.is_empty() {
                    println!("{}", resp.name);
                }
            }
        }
        Ok(())
    })
    .await
}
