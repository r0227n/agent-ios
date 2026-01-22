//! Session state management
//!
//! This module handles reading and writing session state files stored in
//! `/tmp/idb/sessions/<session_name>/session.json`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::helpers::CommandResult;

/// Default sessions directory path
const SESSIONS_DIR: &str = "/tmp/idb/sessions";

/// Session data stored in JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session name
    pub name: String,
    /// Target device UDID
    pub udid: String,
    /// Platform (ios or android)
    pub platform: String,
    /// Current active app bundle_id
    pub app: Option<String>,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Last snapshot information
    pub last_snapshot: Option<SnapshotInfo>,
}

/// Snapshot information stored in session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// Snapshot identifier
    pub snapshot_id: String,
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Reference count (number of elements with refs)
    pub ref_count: usize,
}

/// Session state manager
pub struct SessionState {
    sessions_dir: PathBuf,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    /// Create a new SessionState with default directory
    pub fn new() -> Self {
        Self {
            sessions_dir: PathBuf::from(SESSIONS_DIR),
        }
    }

    /// Create a new SessionState with custom directory (for testing)
    #[allow(dead_code)]
    pub fn with_dir<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            sessions_dir: dir.as_ref().to_path_buf(),
        }
    }

    /// Get the path to a session's JSON file
    fn session_path(&self, name: &str) -> PathBuf {
        self.sessions_dir.join(name).join("session.json")
    }

    /// List all active sessions
    pub fn list_sessions(&self) -> CommandResult<Vec<SessionData>> {
        let mut sessions = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }

        let entries = fs::read_dir(&self.sessions_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let session_file = path.join("session.json");
                if session_file.exists() {
                    match self.read_session_file(&session_file) {
                        Ok(data) => sessions.push(data),
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to read session file {:?}: {}",
                                session_file, e
                            );
                        }
                    }
                }
            }
        }

        // Sort by last activity (most recent first)
        sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

        Ok(sessions)
    }

    /// Get a specific session by name
    pub fn get_session(&self, name: &str) -> CommandResult<Option<SessionData>> {
        let path = self.session_path(name);

        if !path.exists() {
            return Ok(None);
        }

        let data = self.read_session_file(&path)?;
        Ok(Some(data))
    }

    /// Create a new session
    pub fn create_session(&self, data: SessionData) -> CommandResult<()> {
        let session_dir = self.sessions_dir.join(&data.name);

        // Check if session already exists
        if session_dir.exists() {
            return Err(format!("Session '{}' already exists", data.name).into());
        }

        // Create session directory
        fs::create_dir_all(&session_dir)?;

        // Write session file
        let path = session_dir.join("session.json");
        self.write_session_file(&path, &data)?;

        Ok(())
    }

    /// Update an existing session
    #[allow(dead_code)]
    pub fn update_session(&self, data: &SessionData) -> CommandResult<()> {
        let path = self.session_path(&data.name);

        if !path.exists() {
            return Err(format!("Session '{}' does not exist", data.name).into());
        }

        self.write_session_file(&path, data)?;
        Ok(())
    }

    /// Update last activity timestamp for a session
    #[allow(dead_code)]
    pub fn touch_session(&self, name: &str) -> CommandResult<()> {
        if let Some(mut data) = self.get_session(name)? {
            data.last_activity = Utc::now();
            self.update_session(&data)?;
        }
        Ok(())
    }

    /// Destroy (delete) a session
    pub fn destroy_session(&self, name: &str) -> CommandResult<()> {
        let session_dir = self.sessions_dir.join(name);

        if !session_dir.exists() {
            return Err(format!("Session '{}' does not exist", name).into());
        }

        fs::remove_dir_all(&session_dir)?;
        Ok(())
    }

    /// Check if a session exists
    pub fn session_exists(&self, name: &str) -> bool {
        self.session_path(name).exists()
    }

    fn read_session_file(&self, path: &Path) -> CommandResult<SessionData> {
        let content = fs::read_to_string(path)?;
        let data: SessionData = serde_json::from_str(&content)?;
        Ok(data)
    }

    fn write_session_file(&self, path: &Path, data: &SessionData) -> CommandResult<()> {
        let content = serde_json::to_string_pretty(data)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_session_data(name: &str, udid: &str) -> SessionData {
        SessionData {
            name: name.to_string(),
            udid: udid.to_string(),
            platform: "ios".to_string(),
            app: None,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            last_snapshot: None,
        }
    }

    #[test]
    fn test_create_and_get_session() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let data = create_test_session_data("test1", "12345-UDID");
        state.create_session(data.clone()).unwrap();

        let retrieved = state.get_session("test1").unwrap().unwrap();
        assert_eq!(retrieved.name, "test1");
        assert_eq!(retrieved.udid, "12345-UDID");
    }

    #[test]
    fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        state
            .create_session(create_test_session_data("session1", "udid1"))
            .unwrap();
        state
            .create_session(create_test_session_data("session2", "udid2"))
            .unwrap();

        let sessions = state.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_destroy_session() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let data = create_test_session_data("to_delete", "udid");
        state.create_session(data).unwrap();
        assert!(state.session_exists("to_delete"));

        state.destroy_session("to_delete").unwrap();
        assert!(!state.session_exists("to_delete"));
    }

    #[test]
    fn test_create_duplicate_session_fails() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let data = create_test_session_data("dup", "udid");
        state.create_session(data.clone()).unwrap();

        let result = state.create_session(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_session() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let result = state.get_session("nonexistent").unwrap();
        assert!(result.is_none());
    }
}
