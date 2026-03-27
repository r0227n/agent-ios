//! Permission management module for Android apps.
//!
//! This module provides functions to grant, revoke, and reset runtime permissions
//! for Android apps using the native ADB protocol.

use tracing::warn;

use super::commands::{AdbError, Result};
use super::connection::AdbConnection;

const PERMISSION_FAILURE_PATTERNS: &[&str] = &["Exception", "Error", "Unknown"];
const RESET_PERMISSION_WARNING_PATTERNS: &[&str] = &["Exception", "Error"];

fn join_blocking_error(error: tokio::task::JoinError) -> AdbError {
    AdbError::CommandFailed(format!("task join error: {}", error))
}

fn trimmed_output_matches_any(output: &str, patterns: &[&str]) -> bool {
    let trimmed = output.trim();
    !trimmed.is_empty() && patterns.iter().any(|pattern| trimmed.contains(pattern))
}

async fn with_connection<T, F>(serial: Option<&str>, f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&mut AdbConnection) -> Result<T> + Send + 'static,
{
    let serial = serial.map(str::to_owned);
    tokio::task::spawn_blocking(move || {
        let mut conn = AdbConnection::for_device(serial.as_deref())?;
        f(&mut conn)
    })
    .await
    .map_err(join_blocking_error)?
}

async fn run_permission_command(
    serial: Option<&str>,
    package: &str,
    permission: &str,
    pm_command: &'static str,
    relation: &'static str,
) -> Result<()> {
    let package = package.to_string();
    let permission = permission.to_string();

    with_connection(serial, move |conn| {
        let output = conn.shell_command_args(&["pm", pm_command, &package, &permission])?;
        if trimmed_output_matches_any(&output, PERMISSION_FAILURE_PATTERNS) {
            return Err(AdbError::CommandFailed(format!(
                "Failed to {} permission '{}' {} '{}': {}",
                pm_command,
                permission,
                relation,
                package,
                output.trim()
            )));
        }

        Ok(())
    })
    .await
}

fn parse_runtime_permissions(stdout: &str) -> Vec<(String, bool)> {
    let mut permissions = Vec::new();
    let mut in_runtime_permissions = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        let is_permission_line = trimmed.starts_with("android.permission.");

        if trimmed.starts_with("runtime permissions:") {
            in_runtime_permissions = true;
            continue;
        }

        if in_runtime_permissions
            && !trimmed.is_empty()
            && (trimmed.ends_with(':')
                || (!is_permission_line && !line.starts_with(' ') && !line.starts_with('\t')))
        {
            break;
        }

        if in_runtime_permissions && is_permission_line {
            if let Some((permission, details)) = trimmed.split_once(':') {
                permissions.push((permission.to_string(), details.contains("granted=true")));
            }
        }
    }

    permissions
}

/// Grant a runtime permission to an app.
///
/// Uses `pm grant <package> <permission>` via native ADB protocol.
pub async fn grant_permission(serial: Option<&str>, package: &str, permission: &str) -> Result<()> {
    run_permission_command(serial, package, permission, "grant", "to").await
}

/// Revoke a runtime permission from an app.
///
/// Uses `pm revoke <package> <permission>` via native ADB protocol.
pub async fn revoke_permission(
    serial: Option<&str>,
    package: &str,
    permission: &str,
) -> Result<()> {
    run_permission_command(serial, package, permission, "revoke", "from").await
}

/// Reset all permissions for an app to their default state.
///
/// Uses `pm reset-permissions <package>` via native ADB protocol.
///
/// # Note
/// This requires Android 6.0 (API 23) or higher.
pub async fn reset_permissions(serial: Option<&str>, package: &str) -> Result<()> {
    let package = package.to_string();

    with_connection(serial, move |conn| {
        let output = conn.shell_command_args(&["pm", "reset-permissions", &package])?;

        if trimmed_output_matches_any(&output, RESET_PERMISSION_WARNING_PATTERNS) {
            // reset-permissions may not be available on all Android versions
            // Treat as non-critical error
            warn!(
                package = %package,
                error = %output.trim(),
                "Failed to reset permissions (may not be available on this Android version)"
            );
        }

        Ok(())
    })
    .await
}

/// List all runtime permissions for a package.
///
/// Uses `dumpsys package <package>` via native ADB protocol and parses the permissions section.
pub async fn list_permissions(serial: Option<&str>, package: &str) -> Result<Vec<(String, bool)>> {
    let package = package.to_string();

    let stdout = with_connection(serial, move |conn| {
        conn.shell_command_args(&["dumpsys", "package", &package])
    })
    .await?;

    Ok(parse_runtime_permissions(&stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trimmed_output_matches_any() {
        assert!(trimmed_output_matches_any(
            "  Error: denied  ",
            PERMISSION_FAILURE_PATTERNS
        ));
        assert!(!trimmed_output_matches_any(
            "   ",
            PERMISSION_FAILURE_PATTERNS
        ));
        assert!(!trimmed_output_matches_any(
            "Success",
            PERMISSION_FAILURE_PATTERNS
        ));
    }

    #[test]
    fn test_parse_runtime_permissions() {
        let stdout = r#"
            requested permissions:
              android.permission.CAMERA
            runtime permissions:
              android.permission.CAMERA: granted=true, flags=[ USER_SENSITIVE_WHEN_GRANTED]
              android.permission.RECORD_AUDIO: granted=false, flags=[ USER_SENSITIVE_WHEN_DENIED]
            install permissions:
              android.permission.INTERNET: granted=true
        "#;

        assert_eq!(
            parse_runtime_permissions(stdout),
            vec![
                ("android.permission.CAMERA".to_string(), true),
                ("android.permission.RECORD_AUDIO".to_string(), false),
            ]
        );
    }
}
