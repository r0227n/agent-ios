use crate::helpers::file_container::{file_container_with_root, DefaultContainer};
use crate::helpers::client::{with_client, CommandResult};

/// Remove files or directories inside a container
pub async fn run(
    paths: Vec<String>,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> CommandResult {
    // Build FileContainer - defaults to Root if no bundle_id specified
    let container = if bundle_id.is_some() {
        Some(file_container_with_root(
            bundle_id,
            false,
            DefaultContainer::Root,
        ))
    } else {
        Some(file_container_with_root(
            None,
            false,
            DefaultContainer::Root,
        ))
    };

    with_client(udid.as_deref(), |mut client| async move {
        client.rm(paths, container).await?;
        Ok(())
    })
    .await
}
