//! Session state management
//!
//! This module handles reading and writing session state files.
//! By default, session files are stored in:
//! - Unix: `/tmp/agent-mobile/sessions/<session_name>/session.json`
//! - Windows: `%TEMP%\agent-mobile\sessions\<session_name>\session.json`
//!
//! The directory can be customized via the `AGENT_MOBILE_SESSIONS_DIR` environment variable.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::helpers::CommandResult;
use crate::types::Platform;

/// Get the default sessions directory path based on the platform.
///
/// Priority:
/// 1. Environment variable `AGENT_MOBILE_SESSIONS_DIR` if set
/// 2. Unix-like systems (macOS, Linux): `/tmp/agent-mobile/sessions`
/// 3. Windows: `%TEMP%\agent-mobile\sessions`
///
/// # Examples
/// ```
/// // Set custom directory via environment variable
/// std::env::set_var("AGENT_MOBILE_SESSIONS_DIR", "/custom/path");
/// let dir = get_default_sessions_dir();
/// assert_eq!(dir, PathBuf::from("/custom/path"));
/// ```
fn get_default_sessions_dir() -> PathBuf {
    // Check environment variable first
    if let Ok(dir) = std::env::var("AGENT_MOBILE_SESSIONS_DIR") {
        return PathBuf::from(dir);
    }

    // Platform-specific defaults
    #[cfg(unix)]
    {
        PathBuf::from("/tmp/agent-mobile/sessions")
    }

    #[cfg(not(unix))]
    {
        let mut path = std::env::temp_dir();
        path.push("agent-mobile");
        path.push("sessions");
        path
    }
}

/// Session data stored in JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session name
    pub name: String,
    /// Target device UDID
    pub udid: String,
    /// Platform (ios or android)
    pub platform: Platform,
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
            sessions_dir: get_default_sessions_dir(),
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
}

/// Validate that a session name is safe to use as a filesystem path component.
///
/// Returns an error if the name:
/// - Contains path separators (/, \)
/// - Contains path traversal sequences (.., .)
/// - Contains invalid characters (only alphanumeric, hyphen, underscore allowed)
/// - Is empty or whitespace-only
/// - Exceeds 255 characters
///
/// # Examples
/// ```
/// validate_session_name("my-session")  // Ok
/// validate_session_name("test_123")    // Ok
/// validate_session_name("../evil")     // Err
/// validate_session_name("")            // Err
/// ```
fn validate_session_name(name: &str) -> CommandResult<()> {
    // Check for empty or whitespace-only names
    if name.trim().is_empty() {
        return Err("Session name cannot be empty".into());
    }

    // Check for path separators or traversal sequences
    if name.contains('/') || name.contains('\\') {
        return Err("Session name cannot contain path separators".into());
    }

    if name == "." || name == ".." || name.contains("..") {
        return Err("Session name cannot contain path traversal sequences".into());
    }

    // Allowlist: only alphanumeric, hyphen, and underscore
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Session name can only contain letters, numbers, hyphens, and underscores".into(),
        );
    }

    // Optional: limit length to prevent filesystem issues
    if name.len() > 255 {
        return Err("Session name too long (max 255 characters)".into());
    }

    Ok(())
}

impl SessionState {
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
        validate_session_name(name)?;

        let path = self.session_path(name);

        if !path.exists() {
            return Ok(None);
        }

        let data = self.read_session_file(&path)?;
        Ok(Some(data))
    }

    /// Create a new session
    pub fn create_session(&self, data: SessionData) -> CommandResult<()> {
        validate_session_name(&data.name)?;

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
        validate_session_name(&data.name)?;

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
        validate_session_name(name)?;

        let mut data = self
            .get_session(name)?
            .ok_or_else(|| format!("Session '{}' does not exist", name))?;

        data.last_activity = Utc::now();
        self.update_session(&data)?;

        Ok(())
    }

    /// Destroy (delete) a session
    pub fn destroy_session(&self, name: &str) -> CommandResult<()> {
        validate_session_name(name)?;

        let session_dir = self.sessions_dir.join(name);

        if !session_dir.exists() {
            return Err(format!("Session '{}' does not exist", name).into());
        }

        fs::remove_dir_all(&session_dir)?;
        Ok(())
    }

    /// Check if a session exists
    pub fn session_exists(&self, name: &str) -> bool {
        // Return false if validation fails (invalid names cannot exist)
        if validate_session_name(name).is_err() {
            return false;
        }
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
            platform: Platform::Ios,
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

    #[test]
    fn test_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let attack_names = vec![
            "../evil",
            "../../etc/passwd",
            ".",
            "..",
            "./session",
            "../../../root",
            "valid/../evil",
        ];

        for name in attack_names {
            let data = create_test_session_data(name, "udid");
            let result = state.create_session(data);
            assert!(result.is_err(), "Should reject: {}", name);
        }
    }

    #[test]
    fn test_valid_session_names() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let valid_names = vec![
            "my-session",
            "test_123",
            "session1",
            "dev-ios-sim",
            "TEST_ANDROID",
        ];

        for name in valid_names {
            let data = create_test_session_data(name, "udid");
            let result = state.create_session(data);
            assert!(result.is_ok(), "Should accept: {}", name);
        }
    }

    #[test]
    fn test_invalid_characters() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let invalid_names = vec![
            "session with spaces",
            "session@special",
            "session!exclaim",
            "session/slash",
            "session\\backslash",
        ];

        for name in invalid_names {
            let data = create_test_session_data(name, "udid");
            let result = state.create_session(data);
            assert!(result.is_err(), "Should reject: {}", name);
        }
    }

    #[test]
    fn test_empty_session_name() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let data = create_test_session_data("", "udid");
        assert!(state.create_session(data).is_err());

        let data = create_test_session_data("   ", "udid");
        assert!(state.create_session(data).is_err());
    }

    #[test]
    fn test_touch_nonexistent_session_fails() {
        let temp_dir = TempDir::new().unwrap();
        let state = SessionState::with_dir(temp_dir.path());

        let result = state.touch_session("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
