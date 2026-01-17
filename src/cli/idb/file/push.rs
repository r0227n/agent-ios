use crate::cli::helpers::{file_container, with_client, CommandResult};

pub async fn run(
    src_path: String,
    dst_path: String,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> CommandResult {
    let container = file_container(bundle_id);

    with_client(udid.as_deref(), |mut client| async move {
        client.push(src_path, dst_path, container).await?;
        Ok(())
    })
    .await
}
