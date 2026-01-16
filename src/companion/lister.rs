use crate::types::{TargetDescription, TargetType};
use serde::Deserialize;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListError {
    #[error("idb_companion not found")]
    CompanionNotFound,

    #[error("Failed to execute idb_companion: {0}")]
    ExecutionFailed(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),
}

/// Response from idb_companion --list 1 (JSON per line)
#[derive(Debug, Deserialize)]
struct CompanionListItem {
    name: String,
    udid: String,
    state: Option<String>,
    #[serde(rename = "type")]
    target_type: String,
    os_version: Option<String>,
    architecture: Option<String>,
}

impl From<CompanionListItem> for TargetDescription {
    fn from(item: CompanionListItem) -> Self {
        // Normalize empty state to None
        let state = item.state.filter(|s| !s.is_empty());

        TargetDescription {
            name: item.name,
            udid: item.udid,
            state,
            target_type: TargetType::from_proto_string(&item.target_type),
            os_version: item.os_version,
            architecture: item.architecture,
            companion_info: None, // Local targets have no companion connection
        }
    }
}

/// Lists local targets using idb_companion --list 1
pub struct CompanionLister {
    companion_path: String,
}

impl CompanionLister {
    /// Create a new lister, finding idb_companion in PATH or common locations
    pub fn new() -> Result<Self, ListError> {
        // Check IDB_COMPANION environment variable first
        if let Ok(path) = std::env::var("IDB_COMPANION") {
            if std::path::Path::new(&path).exists() {
                return Ok(Self {
                    companion_path: path,
                });
            }
        }

        // Try to find idb_companion using `which`
        if let Ok(output) = Command::new("which").arg("idb_companion").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Self {
                        companion_path: path,
                    });
                }
            }
        }

        // Check common installation locations
        let common_paths = [
            "/usr/local/bin/idb_companion",
            "/opt/homebrew/bin/idb_companion",
        ];

        for path in &common_paths {
            if std::path::Path::new(path).exists() {
                return Ok(Self {
                    companion_path: path.to_string(),
                });
            }
        }

        Err(ListError::CompanionNotFound)
    }

    /// List all local targets using idb_companion --list 1
    pub fn list_targets(
        &self,
        only: Option<TargetType>,
    ) -> Result<Vec<TargetDescription>, ListError> {
        let mut cmd = Command::new(&self.companion_path);
        cmd.arg("--list").arg("1");

        // Apply filter
        if let Some(filter) = only {
            match filter {
                TargetType::Simulator => {
                    cmd.arg("--only").arg("simulator");
                }
                TargetType::Device => {
                    cmd.arg("--only").arg("device");
                }
                TargetType::Mac => {
                    // No filter for mac
                }
            }
        }

        let output = cmd
            .output()
            .map_err(|e| ListError::ExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(ListError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut targets = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let item: CompanionListItem =
                serde_json::from_str(line).map_err(|e| ListError::ParseError(e.to_string()))?;
            targets.push(item.into());
        }

        Ok(targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_companion_list_item() {
        let json = r#"{"name": "iPhone 14 Pro", "udid": "ABC-123", "state": "Booted", "type": "simulator", "os_version": "iOS 17.0", "architecture": "arm64"}"#;
        let item: CompanionListItem = serde_json::from_str(json).unwrap();

        assert_eq!(item.name, "iPhone 14 Pro");
        assert_eq!(item.udid, "ABC-123");
        assert_eq!(item.state, Some("Booted".to_string()));
        assert_eq!(item.target_type, "simulator");
    }

    #[test]
    fn test_companion_list_item_to_target_description() {
        let json = r#"{"name": "iPhone 14 Pro", "udid": "ABC-123", "state": "Booted", "type": "simulator", "os_version": "iOS 17.0", "architecture": "arm64"}"#;
        let item: CompanionListItem = serde_json::from_str(json).unwrap();
        let target: TargetDescription = item.into();

        assert_eq!(target.name, "iPhone 14 Pro");
        assert_eq!(target.udid, "ABC-123");
        assert_eq!(target.state, Some("Booted".to_string()));
        assert_eq!(target.target_type, TargetType::Simulator);
        assert!(target.companion_info.is_none());
    }

    #[test]
    fn test_empty_state_normalized_to_none() {
        let json = r#"{"name": "iPhone", "udid": "ABC", "state": "", "type": "simulator"}"#;
        let item: CompanionListItem = serde_json::from_str(json).unwrap();
        let target: TargetDescription = item.into();

        assert!(target.state.is_none());
    }
}
