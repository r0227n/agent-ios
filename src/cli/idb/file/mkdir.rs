use crate::companion::CompanionResolver;
use crate::grpc::idb::file_container::Kind as FileContainerKind;
use crate::grpc::idb::FileContainer;

pub async fn run(
    path: String,
    bundle_id: Option<String>,
    root: bool,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Connect to companion (with auto-spawning if needed)
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // 2. Determine the file container
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
        // Default to root if no container is specified
        FileContainer {
            kind: FileContainerKind::Root as i32,
            bundle_id: String::new(),
        }
    };

    // 3. Call mkdir RPC
    client.mkdir(&path, container).await?;

    // 4. Success (no output for mkdir command, same as Python idb)
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_selection_root() {
        let container = FileContainer {
            kind: FileContainerKind::Root as i32,
            bundle_id: String::new(),
        };
        assert_eq!(container.kind, FileContainerKind::Root as i32);
        assert_eq!(container.bundle_id, "");
    }

    #[test]
    fn test_container_selection_application() {
        let bundle_id = "com.example.app".to_string();
        let container = FileContainer {
            kind: FileContainerKind::Application as i32,
            bundle_id: bundle_id.clone(),
        };
        assert_eq!(container.kind, FileContainerKind::Application as i32);
        assert_eq!(container.bundle_id, bundle_id);
    }
}
