//! terminate command - Terminate a running application.
//!
//! Stops a running app by its bundle identifier.

use crate::helpers::client::{with_client, CommandResult};

/// Execute the terminate command.
pub async fn run(bundle_id: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.terminate(&bundle_id).await?;
        Ok(())
    })
    .await
}
