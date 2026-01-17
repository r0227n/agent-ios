use crate::cli::helpers::{file_container_with_root, with_client, CommandResult, DefaultContainer};

pub async fn run(
    path: String,
    bundle_id: Option<String>,
    root: bool,
    udid: Option<String>,
) -> CommandResult {
    let container = file_container_with_root(bundle_id, root, DefaultContainer::Root);

    with_client(udid.as_deref(), |mut client| async move {
        client.mkdir(&path, container).await?;
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use crate::cli::helpers::{file_container_with_root, DefaultContainer};
    use crate::grpc::idb::file_container::Kind as FileContainerKind;

    #[test]
    fn test_container_selection_root() {
        let container = file_container_with_root(None, false, DefaultContainer::Root);
        assert_eq!(container.kind, FileContainerKind::Root as i32);
        assert_eq!(container.bundle_id, "");
    }

    #[test]
    fn test_container_selection_application() {
        let container = file_container_with_root(
            Some("com.example.app".to_string()),
            false,
            DefaultContainer::Root,
        );
        assert_eq!(container.kind, FileContainerKind::Application as i32);
        assert_eq!(container.bundle_id, "com.example.app");
    }
}
