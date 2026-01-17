use crate::companion::CompanionResolver;
use crate::grpc::idb::file_container::Kind as FileContainerKind;
use crate::grpc::idb::FileContainer;

pub async fn run(
    paths: Vec<String>,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Connect to companion
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // 2. Build file container
    let container = bundle_id.map(|bundle_id| FileContainer {
        kind: FileContainerKind::Application as i32,
        bundle_id,
    });

    // 3. Call ls RPC
    let response = if paths.len() == 1 {
        // Single path - use simpler output format (just file names)
        client.ls(paths[0].clone(), vec![], container).await?
    } else {
        // Multiple paths - use listings format (path: files...)
        client.ls(String::new(), paths.clone(), container).await?
    };

    // 4. Format and output
    if paths.len() == 1 && !response.files.is_empty() {
        // Single path output - just print file paths
        for file_info in response.files {
            println!("{}", file_info.path);
        }
    } else if !response.listings.is_empty() {
        // Multiple paths output - print with directory headers
        for (idx, listing) in response.listings.iter().enumerate() {
            if let Some(parent) = &listing.parent {
                println!("{}:", parent.path);
            }
            for file_info in &listing.files {
                println!("{}", file_info.path);
            }
            // Add blank line between listings (except after the last one)
            if idx < response.listings.len() - 1 {
                println!();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_container_creation() {
        // Test with bundle_id
        let bundle_id = Some("com.example.app".to_string());
        let container = bundle_id.map(|bundle_id| FileContainer {
            kind: FileContainerKind::Application as i32,
            bundle_id,
        });
        assert!(container.is_some());
        let container = container.unwrap();
        assert_eq!(container.kind, FileContainerKind::Application as i32);
        assert_eq!(container.bundle_id, "com.example.app");

        // Test without bundle_id
        let bundle_id: Option<String> = None;
        let container = bundle_id.map(|bundle_id| FileContainer {
            kind: FileContainerKind::Application as i32,
            bundle_id,
        });
        assert!(container.is_none());
    }

    #[test]
    fn test_single_vs_multiple_paths() {
        let single_path = ["/path/to/file".to_string()];
        assert_eq!(single_path.len(), 1);

        let multiple_paths = ["/path/to/file1".to_string(), "/path/to/file2".to_string()];
        assert!(multiple_paths.len() > 1);
    }
}
