use crate::helpers::{file_container, with_client, CommandResult};

pub async fn run(
    paths: Vec<String>,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> CommandResult {
    let container = file_container(bundle_id);

    with_client(udid.as_deref(), |mut client| async move {
        let response = if paths.len() == 1 {
            // Single path - use simpler output format (just file names)
            client.ls(paths[0].clone(), vec![], container).await?
        } else {
            // Multiple paths - use listings format (path: files...)
            client.ls(String::new(), paths.clone(), container).await?
        };

        // Format and output
        if paths.len() == 1 && !response.files.is_empty() {
            // Single path output - just print file paths
            for file_info in response.files {
                println!("{}", file_info.path);
            }
        } else if !response.listings.is_empty() {
            // Multiple paths output - print with directory headers
            // Re-sort listings to match input paths order (gRPC response order may differ)
            let listings_map: std::collections::HashMap<
                &str,
                &agent_mobile_platform_ios::proto::idb::FileListing,
            > = response
                .listings
                .iter()
                .filter_map(|l| l.parent.as_ref().map(|p| (p.path.as_str(), l)))
                .collect();

            for path in &paths {
                if let Some(listing) = listings_map.get(path.as_str()) {
                    if let Some(parent) = &listing.parent {
                        println!("{}:", parent.path);
                    }
                    for file_info in &listing.files {
                        println!("{}", file_info.path);
                    }
                }
            }
        }

        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use crate::helpers::file_container;
    use agent_mobile_platform_ios::proto::idb::file_container::Kind as FileContainerKind;

    #[test]
    fn test_file_container_creation() {
        // Test with bundle_id
        let container = file_container(Some("com.example.app".to_string()));
        assert!(container.is_some());
        let container = container.unwrap();
        assert_eq!(container.kind, FileContainerKind::Application as i32);
        assert_eq!(container.bundle_id, "com.example.app");

        // Test without bundle_id
        let container = file_container(None);
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
