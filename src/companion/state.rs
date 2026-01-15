use crate::types::Address;
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
                serde_json::from_str(&contents).unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    }

    /// Find a companion by UDID
    pub fn find_by_udid(&self, udid: &str) -> Option<StoredCompanion> {
        self.get_companions().into_iter().find(|c| c.udid == udid)
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

    #[test]
    fn test_empty_state_file() {
        let file = NamedTempFile::new().unwrap();

        let state = CompanionState::new(file.path().to_str().unwrap());
        let companions = state.get_companions();

        assert!(companions.is_empty());
    }

    #[test]
    fn test_nonexistent_state_file() {
        let state = CompanionState::new("/nonexistent/path/state");
        let companions = state.get_companions();

        assert!(companions.is_empty());
    }

    #[test]
    fn test_find_by_udid() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"[
                {{"udid": "ABC123", "is_local": true, "path": "/tmp/idb/abc.sock"}},
                {{"udid": "XYZ789", "is_local": false, "host": "localhost", "port": 9888}}
            ]"#
        )
        .unwrap();

        let state = CompanionState::new(file.path().to_str().unwrap());

        // Found
        let found = state.find_by_udid("ABC123");
        assert!(found.is_some());
        assert_eq!(found.unwrap().udid, "ABC123");

        // Found second one
        let found2 = state.find_by_udid("XYZ789");
        assert!(found2.is_some());
        assert_eq!(found2.unwrap().udid, "XYZ789");

        // Not found
        let not_found = state.find_by_udid("NOTEXIST");
        assert!(not_found.is_none());
    }
}
