use crate::companion::CompanionResolver;
use crate::grpc::idb::file_container::Kind as FileContainerKind;
use crate::grpc::idb::FileContainer;

pub async fn run(
    src_path: String,
    dst_path: String,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let container = bundle_id.map(|bundle_id| FileContainer {
        kind: FileContainerKind::Application as i32,
        bundle_id,
    });

    client.push(src_path, dst_path, container).await?;

    Ok(())
}
