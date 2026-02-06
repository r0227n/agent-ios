//! Session resolver for UDID resolution from session names
//!
//! This module handles resolving a session name to a device UDID,
//! with fallback to explicit UDID or auto-detection.

use super::state::SessionState;
use crate::helpers::client::CommandResult;

/// Session resolver for determining target device UDID
pub struct SessionResolver {
    state: SessionState,
}

impl Default for SessionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionResolver {
    /// Create a new SessionResolver
    pub fn new() -> Self {
        Self {
            state: SessionState::new(),
        }
    }

    /// Resolve UDID from session name, explicit UDID, or None for auto-detection.
    ///
    /// Priority:
    /// 1. If session is specified, use session's UDID
    /// 2. If explicit UDID is specified, use it
    /// 3. Return None (auto-detection via simctl)
    ///
    /// # Arguments
    /// * `session` - Optional session name
    /// * `explicit_udid` - Optional explicitly specified UDID
    ///
    /// # Returns
    /// * `Ok(Some(udid))` - Resolved UDID
    /// * `Ok(None)` - No UDID specified, auto-detection will be used
    /// * `Err` - Session not found or other error
    pub fn resolve_udid(
        &self,
        session: Option<&str>,
        explicit_udid: Option<&str>,
    ) -> CommandResult<Option<String>> {
        // Priority 1: Session UDID
        if let Some(session_name) = session {
            let session_data = self
                .state
                .get_session(session_name)?
                .ok_or_else(|| format!("Session '{}' not found", session_name))?;

            // Touch the session to update last_activity
            self.state.touch_session(session_name)?;

            return Ok(Some(session_data.udid));
        }

        // Priority 2: Explicit UDID
        if let Some(udid) = explicit_udid {
            return Ok(Some(udid.to_string()));
        }

        // Priority 3: None (auto-detection via simctl)
        Ok(None)
    }

    /// Get session data if session is specified
    #[allow(dead_code)]
    pub fn get_session_data(
        &self,
        session: Option<&str>,
    ) -> CommandResult<Option<super::state::SessionData>> {
        match session {
            Some(name) => self.state.get_session(name),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::state::SessionData;
    use agent_mobile_core::Platform;
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_resolver(temp_dir: &TempDir) -> SessionResolver {
        SessionResolver {
            state: SessionState::with_dir(temp_dir.path()),
        }
    }

    fn create_test_session(name: &str, udid: &str) -> SessionData {
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
    fn test_resolve_from_session() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = create_test_resolver(&temp_dir);

        // Create a session
        resolver
            .state
            .create_session(create_test_session("mysession", "session-udid-123"))
            .unwrap();

        // Resolve from session
        let udid = resolver
            .resolve_udid(Some("mysession"), None)
            .unwrap()
            .unwrap();
        assert_eq!(udid, "session-udid-123");
    }

    #[test]
    fn test_resolve_explicit_udid() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = create_test_resolver(&temp_dir);

        // Resolve from explicit UDID
        let udid = resolver
            .resolve_udid(None, Some("explicit-udid"))
            .unwrap()
            .unwrap();
        assert_eq!(udid, "explicit-udid");
    }

    #[test]
    fn test_resolve_none() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = create_test_resolver(&temp_dir);

        // No session, no explicit UDID
        let udid = resolver.resolve_udid(None, None).unwrap();
        assert!(udid.is_none());
    }

    #[test]
    fn test_session_priority_over_explicit() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = create_test_resolver(&temp_dir);

        // Create a session
        resolver
            .state
            .create_session(create_test_session("mysession", "session-udid"))
            .unwrap();

        // Session takes priority over explicit UDID
        let udid = resolver
            .resolve_udid(Some("mysession"), Some("explicit-udid"))
            .unwrap()
            .unwrap();
        assert_eq!(udid, "session-udid");
    }

    #[test]
    fn test_nonexistent_session_error() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = create_test_resolver(&temp_dir);

        let result = resolver.resolve_udid(Some("nonexistent"), None);
        assert!(result.is_err());
    }
}
