//! Persistent snapshot cache for ref reuse across CLI invocations.

use chrono::Utc;
use std::fs;
use std::path::PathBuf;

use crate::helpers::client::CommandResult;

use super::types::Snapshot;

fn cache_dir() -> PathBuf {
    if let Ok(path) = std::env::var("AGENT_MOBILE_SNAPSHOT_CACHE_DIR") {
        return PathBuf::from(path);
    }

    #[cfg(unix)]
    {
        PathBuf::from("/tmp/agent-mobile/snapshot-cache")
    }

    #[cfg(not(unix))]
    {
        let mut path = std::env::temp_dir();
        path.push("agent-mobile");
        path.push("snapshot-cache");
        path
    }
}

fn sanitize_component(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn cache_key(udid: &str) -> String {
    if let Ok(session) = std::env::var("AGENT_MOBILE_SESSION") {
        format!("session-{}", sanitize_component(&session))
    } else {
        format!("device-{}", sanitize_component(udid))
    }
}

fn cache_path(udid: &str) -> PathBuf {
    cache_dir().join(format!("{}.json", cache_key(udid)))
}

pub fn load_snapshot_cache(udid: &str) -> CommandResult<Option<Snapshot>> {
    let path = cache_path(udid);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let snapshot: Snapshot = serde_json::from_str(&content)?;
    Ok(Some(snapshot))
}

pub fn save_snapshot_cache(udid: &str, snapshot: &Snapshot) -> CommandResult {
    let dir = cache_dir();
    fs::create_dir_all(&dir)?;
    let path = cache_path(udid);
    let temp_path = dir.join(format!(".{}.{}.tmp", cache_key(udid), nanoid::nanoid!(8)));
    fs::write(&temp_path, serde_json::to_string_pretty(snapshot)?)?;

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(&path)?;
    }

    fs::rename(&temp_path, &path)?;
    update_session_last_snapshot(snapshot)?;
    Ok(())
}

fn update_session_last_snapshot(snapshot: &Snapshot) -> CommandResult {
    let Ok(session_name) = std::env::var("AGENT_MOBILE_SESSION") else {
        return Ok(());
    };

    let state = crate::session::state::SessionState::new();
    if state.get_session(&session_name)?.is_none() {
        return Ok(());
    }

    state.update_session_mut(&session_name, |session| {
        session.last_snapshot = Some(crate::session::state::SnapshotInfo {
            snapshot_id: snapshot.snapshot_id.clone(),
            timestamp: snapshot.timestamp,
            ref_count: snapshot.elements.len(),
        });
        session.last_activity = Utc::now();
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::types::{Snapshot, SnapshotElement};
    use agent_mobile_core::snapshot::Frame;
    use tempfile::TempDir;

    fn sample_snapshot() -> Snapshot {
        Snapshot {
            snapshot_id: "snap_test".to_string(),
            timestamp: Utc::now(),
            active_bundle_id: Some("com.example.app".to_string()),
            snapshot_generation: Some(7),
            elements: vec![SnapshotElement {
                ref_id: "@e1".to_string(),
                element_id: Some("button|Login".to_string()),
                element_type: "Button".to_string(),
                label: Some("Login".to_string()),
                frame: Frame::zero(),
                enabled: true,
                traits: vec![],
                placeholder: None,
                value: None,
                children_indices: vec![],
                depth: 0,
                is_interactive: true,
                parent_index: None,
            }],
        }
    }

    #[test]
    fn test_sanitize_component() {
        assert_eq!(sanitize_component("ABC-123_def"), "ABC-123_def");
        assert_eq!(sanitize_component("session:name"), "session_name");
    }

    #[test]
    fn test_cache_round_trip_serde() {
        let _tmp = TempDir::new().unwrap();
        let snapshot = sample_snapshot();
        let encoded = serde_json::to_string(&snapshot).unwrap();
        let decoded: Snapshot = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.snapshot_id, snapshot.snapshot_id);
        assert_eq!(decoded.snapshot_generation, snapshot.snapshot_generation);
        assert_eq!(
            decoded.elements[0].element_id,
            snapshot.elements[0].element_id
        );
    }
}
