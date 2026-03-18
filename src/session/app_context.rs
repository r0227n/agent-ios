//! Persisted app context for iOS automation commands.
//!
//! Stores the last app launched per device so later XCUITest-backed commands
//! can restore the intended foreground app after the runner starts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::helpers::client::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceAppContext {
    bundle_id: String,
    updated_at: DateTime<Utc>,
}

fn app_context_dir() -> PathBuf {
    if let Ok(path) = std::env::var("AGENT_MOBILE_APP_CONTEXT_DIR") {
        return PathBuf::from(path);
    }

    #[cfg(unix)]
    {
        PathBuf::from("/tmp/agent-mobile/app-context")
    }

    #[cfg(not(unix))]
    {
        let mut path = std::env::temp_dir();
        path.push("agent-mobile");
        path.push("app-context");
        path
    }
}

fn sanitize_udid(udid: &str) -> String {
    udid.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn app_context_path(udid: &str) -> PathBuf {
    app_context_dir().join(format!("{}.json", sanitize_udid(udid)))
}

fn update_session_app(bundle_id: Option<&str>) -> CommandResult {
    let Some(session_name) = std::env::var("AGENT_MOBILE_SESSION").ok() else {
        return Ok(());
    };

    let state = super::state::SessionState::new();
    if state.get_session(&session_name)?.is_none() {
        return Ok(());
    }

    state.update_session_mut(&session_name, |session| {
        session.app = bundle_id.map(|bundle_id| bundle_id.to_string());
        session.last_activity = Utc::now();
    })?;
    Ok(())
}

pub fn set_active_app(udid: &str, bundle_id: &str) -> CommandResult {
    fs::create_dir_all(app_context_dir())?;

    let context = DeviceAppContext {
        bundle_id: bundle_id.to_string(),
        updated_at: Utc::now(),
    };
    let path = app_context_path(udid);
    let json = serde_json::to_vec_pretty(&context)?;
    fs::write(path, json)?;
    update_session_app(Some(bundle_id))?;
    Ok(())
}

pub fn get_active_app(udid: &str) -> CommandResult<Option<String>> {
    let path = app_context_path(udid);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read(&path)?;
    let context: DeviceAppContext = serde_json::from_slice(&raw)?;
    Ok(Some(context.bundle_id))
}

pub fn clear_active_app(udid: &str) -> CommandResult {
    let path = app_context_path(udid);
    if path.exists() {
        fs::remove_file(path)?;
    }
    update_session_app(None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_udid_preserves_safe_chars() {
        assert_eq!(sanitize_udid("ABC-123_def"), "ABC-123_def");
    }

    #[test]
    fn test_sanitize_udid_replaces_unsafe_chars() {
        assert_eq!(sanitize_udid("emulator-5554:tcp"), "emulator-5554_tcp");
    }
}
