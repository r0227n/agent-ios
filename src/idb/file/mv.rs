use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::file_container::{file_container_with_root, DefaultContainer};

pub async fn run(
    src_paths: Vec<String>,
    dst_path: String,
    bundle_id: Option<String>,
    root: bool,
    udid: Option<String>,
) -> CommandResult {
    // Validate input
    if src_paths.is_empty() {
        return Err("At least one source path is required".into());
    }

    let container = file_container_with_root(bundle_id, root, DefaultContainer::Media);

    with_client(udid.as_deref(), |mut client| async move {
        client.mv(src_paths, &dst_path, container).await?;
        Ok(())
    })
    .await
}
