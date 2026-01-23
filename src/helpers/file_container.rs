//! FileContainer builder helpers for file commands
//!
//! This module provides utilities for building FileContainer
//! from CLI arguments (bundle_id, root flags).

use agent_mobile_platform_ios::proto::idb::file_container::Kind as FileContainerKind;
use agent_mobile_platform_ios::proto::idb::FileContainer;

/// Default container kind when no explicit container is specified
#[derive(Clone, Copy, Default)]
pub enum DefaultContainer {
    /// Media container (default for most file operations)
    #[default]
    Media,
    /// Root container
    Root,
}

/// Build a FileContainer from a bundle_id option.
///
/// Returns `Some(FileContainer)` if bundle_id is provided,
/// otherwise returns `None`.
///
/// Use this for commands that only accept `--bundle-id` flag.
///
/// # Example
///
/// ```ignore
/// use crate::helpers::file_container;
///
/// let container = file_container(bundle_id);
/// client.ls(path, vec![], container).await?;
/// ```
pub fn file_container(bundle_id: Option<String>) -> Option<FileContainer> {
    bundle_id.map(|bundle_id| FileContainer {
        kind: FileContainerKind::Application as i32,
        bundle_id,
    })
}

/// Build a FileContainer from bundle_id and root flags with a default.
///
/// Priority:
/// 1. If `bundle_id` is Some, use APPLICATION container
/// 2. If `root` is true, use ROOT container
/// 3. Otherwise use the specified `default` container
///
/// # Example
///
/// ```ignore
/// use crate::helpers::{file_container_with_root, DefaultContainer};
///
/// let container = file_container_with_root(bundle_id, root, DefaultContainer::Media);
/// client.mkdir(&path, container).await?;
/// ```
pub fn file_container_with_root(
    bundle_id: Option<String>,
    root: bool,
    default: DefaultContainer,
) -> FileContainer {
    if let Some(bid) = bundle_id {
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
        match default {
            DefaultContainer::Media => FileContainer {
                kind: FileContainerKind::Media as i32,
                bundle_id: String::new(),
            },
            DefaultContainer::Root => FileContainer {
                kind: FileContainerKind::Root as i32,
                bundle_id: String::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_container_with_bundle_id() {
        let container = file_container(Some("com.example.app".to_string()));
        assert!(container.is_some());
        let container = container.unwrap();
        assert_eq!(container.kind, FileContainerKind::Application as i32);
        assert_eq!(container.bundle_id, "com.example.app");
    }

    #[test]
    fn test_file_container_without_bundle_id() {
        let container = file_container(None);
        assert!(container.is_none());
    }

    #[test]
    fn test_file_container_with_root_bundle_id_priority() {
        // bundle_id takes precedence over root flag
        let container = file_container_with_root(
            Some("com.example.app".to_string()),
            true,
            DefaultContainer::Media,
        );
        assert_eq!(container.kind, FileContainerKind::Application as i32);
        assert_eq!(container.bundle_id, "com.example.app");
    }

    #[test]
    fn test_file_container_with_root_flag() {
        let container = file_container_with_root(None, true, DefaultContainer::Media);
        assert_eq!(container.kind, FileContainerKind::Root as i32);
        assert_eq!(container.bundle_id, "");
    }

    #[test]
    fn test_file_container_default_media() {
        let container = file_container_with_root(None, false, DefaultContainer::Media);
        assert_eq!(container.kind, FileContainerKind::Media as i32);
        assert_eq!(container.bundle_id, "");
    }

    #[test]
    fn test_file_container_default_root() {
        let container = file_container_with_root(None, false, DefaultContainer::Root);
        assert_eq!(container.kind, FileContainerKind::Root as i32);
        assert_eq!(container.bundle_id, "");
    }
}
