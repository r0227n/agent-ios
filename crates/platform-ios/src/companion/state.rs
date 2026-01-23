//! State file management for companion daemons.
//!
//! The idb_companion stores state in /tmp/idb/state as JSON.

use agent_mobile_core::types::Address;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const IDB_STATE_FILE_PATH: &str = "/tmp/idb/state";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCompanion {
    pub udid: String,
    pub is_local: bool,
    #[serde(default)]
    pub pid: Option<u32>,
    // TCP address fields
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    // Domain socket field
    #[serde(default)]
    pub path: Option<String>,
}

impl StoredCompanion {
    pub fn address(&self) -> Option<Address> {
        if let Some(path) = &self.path {
            Some(Address::DomainSocket { path: path.clone() })
        } else if let (Some(host), Some(port)) = (&self.host, self.port) {
            Some(Address::Tcp {
                host: host.clone(),
                port,
            })
        } else {
            None
        }
    }
}

pub struct CompanionState {
    state_file_path: String,
}

impl Default for CompanionState {
    fn default() -> Self {
        Self {
            state_file_path: IDB_STATE_FILE_PATH.to_string(),
        }
    }
}

impl CompanionState {
    #[allow(dead_code)]
    pub fn new(state_file_path: &str) -> Self {
        Self {
            state_file_path: state_file_path.to_string(),
        }
    }

    /// Read stored companions from state file
    pub fn get_companions(&self) -> Vec<StoredCompanion> {
        let path = Path::new(&self.state_file_path);

        if !path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(path) {
            Ok(contents) => {
                if contents.trim().is_empty() {
                    return Vec::new();
                }
                let mut companions: Vec<StoredCompanion> =
                    serde_json::from_str(&contents).unwrap_or_default();
                // Sort by UDID to match Python idb behavior
                companions.sort_by(|a, b| a.udid.cmp(&b.udid));
                companions
            }
            Err(_) => Vec::new(),
        }
    }

    /// Find a companion by UDID
    #[allow(dead_code)]
    pub fn find_by_udid(&self, udid: &str) -> Option<StoredCompanion> {
        self.get_companions().into_iter().find(|c| c.udid == udid)
    }

    /// Clear the state file (delete all stored companions)
    pub fn clear(&self) -> Result<(), std::io::Error> {
        let path = Path::new(&self.state_file_path);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Add or update a companion in the state file
    pub fn add_companion(&self, companion: StoredCompanion) -> Result<(), std::io::Error> {
        // Ensure /tmp/idb directory exists
        if let Some(parent) = Path::new(&self.state_file_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Read existing companions
        let mut companions = self.get_companions();

        // Remove existing entry with same UDID (if any)
        companions.retain(|c| c.udid != companion.udid);

        // Add new companion
        companions.push(companion);

        // Write back to state file
        let contents = serde_json::to_string(&companions)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&self.state_file_path, contents)?;

        Ok(())
    }

    /// Remove a companion by UDID
    #[allow(dead_code)]
    pub fn remove_by_udid(&self, udid: &str) -> Result<Option<StoredCompanion>, std::io::Error> {
        let mut companions = self.get_companions();
        let removed = companions
            .iter()
            .position(|c| c.udid == udid)
            .map(|i| companions.remove(i));

        if removed.is_some() {
            let contents = serde_json::to_string(&companions)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            fs::write(&self.state_file_path, contents)?;
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_domain_socket_companion() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[{{"udid": "ABC123", "is_local": true, "path": "/tmp/idb/test.sock"}}]"#
        )
        .unwrap();

        let state = CompanionState::new(file.path().to_str().unwrap());
        let companions = state.get_companions();

        assert_eq!(companions.len(), 1);
        assert_eq!(companions[0].udid, "ABC123");
        assert!(companions[0].is_local);
        assert_eq!(
            companions[0].address(),
            Some(Address::DomainSocket {
                path: "/tmp/idb/test.sock".to_string()
            })
        );
    }

    #[test]
    fn test_parse_tcp_companion() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[{{"udid": "XYZ789", "is_local": false, "host": "localhost", "port": 9888}}]"#
        )
        .unwrap();

        let state = CompanionState::new(file.path().to_str().unwrap());
        let companions = state.get_companions();

        assert_eq!(companions.len(), 1);
        assert_eq!(companions[0].udid, "XYZ789");
        assert!(!companions[0].is_local);
        assert_eq!(
            companions[0].address(),
            Some(Address::Tcp {
                host: "localhost".to_string(),
                port: 9888
            })
        );
    }
}
