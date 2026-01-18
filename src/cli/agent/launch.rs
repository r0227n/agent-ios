//! AI-optimized app launch command.

use crate::cli::helpers::{with_client, CommandResult};
use crate::grpc::LaunchConfig;
use std::collections::HashMap;
use tokio::sync::watch;

/// Launch an application by bundle ID.
pub async fn run(bundle_id: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let config = LaunchConfig {
            bundle_id,
            app_args: vec![],
            env: HashMap::new(),
            foreground_if_running: true,
            wait_for_debugger: false,
        };

        let (_stop_tx, stop_rx) = watch::channel(false);
        client.launch(config, false, stop_rx).await?;
        Ok(())
    })
    .await
}
