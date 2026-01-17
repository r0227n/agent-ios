use crate::companion::CompanionResolver;
use crate::grpc::idb::file_container::Kind as FileContainerKind;
use crate::grpc::idb::FileContainer;

pub async fn run(
    src_paths: Vec<String>,
    dst_path: String,
    bundle_id: Option<String>,
    root: bool,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Validate input
    if src_paths.is_empty() {
        return Err("At least one source path is required".into());
    }

    // 1. Connect to companion
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // 2. Build FileContainer based on flags
    let container = if let Some(bid) = bundle_id {
        FileContainer {
            kind: FileContainerKind::Application as i32,
            bundle_id: bid,
        }
    } else if root {
        FileContainer {
            kind: FileContainerKind::Root as i32,
            bundle_id: String::new(),
        }
    } else {
        // Default to media
        FileContainer {
            kind: FileContainerKind::Media as i32,
            bundle_id: String::new(),
        }
    };

    // 3. Call mv RPC
    client.mv(src_paths, &dst_path, container).await?;

    // 4. Success (no output on success, matching Python idb behavior)
    Ok(())
}
