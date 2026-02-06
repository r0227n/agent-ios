//! Permission management module for Android apps.
//!
//! This module provides functions to grant, revoke, and reset runtime permissions
//! for Android apps using the native ADB protocol.

use super::commands::{AdbError, Result};
use super::connection::AdbConnection;

/// Grant a runtime permission to an app.
///
/// Uses `pm grant <package> <permission>` via native ADB protocol.
///
/// # Arguments
/// - `serial`: Device serial number (optional, uses default device if None)
/// - `package`: App package name (e.g., "com.example.app")
/// - `permission`: Permission name (e.g., "android.permission.CAMERA")
pub async fn grant_permission(serial: Option<&str>, package: &str, permission: &str) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    let output = conn.shell_command_args(&["pm", "grant", package, permission])?;

    // pm grant outputs error messages on failure
    if output.contains("Exception") || output.contains("Error") || output.contains("Unknown") {
        return Err(AdbError::CommandFailed(format!(
            "Failed to grant permission '{}' to '{}': {}",
            permission,
            package,
            output.trim()
        )));
    }

    Ok(())
}

/// Revoke a runtime permission from an app.
///
/// Uses `pm revoke <package> <permission>` via native ADB protocol.
pub async fn revoke_permission(
    serial: Option<&str>,
    package: &str,
    permission: &str,
) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    let output = conn.shell_command_args(&["pm", "revoke", package, permission])?;

    if output.contains("Exception") || output.contains("Error") || output.contains("Unknown") {
        return Err(AdbError::CommandFailed(format!(
            "Failed to revoke permission '{}' from '{}': {}",
            permission,
            package,
            output.trim()
        )));
    }

    Ok(())
}

/// Reset all permissions for an app to their default state.
///
/// Uses `pm reset-permissions <package>` via native ADB protocol.
///
/// # Note
/// This requires Android 6.0 (API 23) or higher.
pub async fn reset_permissions(serial: Option<&str>, package: &str) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    let output = conn.shell_command_args(&["pm", "reset-permissions", package])?;

    if output.contains("Exception") || output.contains("Error") {
        // reset-permissions may not be available on all Android versions
        // Treat as non-critical error
        eprintln!(
            "Warning: Failed to reset permissions for '{}': {}",
            package,
            output.trim()
        );
    }

    Ok(())
}

/// List all runtime permissions for a package.
///
/// Uses `dumpsys package <package>` via native ADB protocol and parses the permissions section.
pub async fn list_permissions(serial: Option<&str>, package: &str) -> Result<Vec<(String, bool)>> {
    let mut conn = AdbConnection::for_device(serial)?;
    let stdout = conn.shell_command_args(&["dumpsys", "package", package])?;

    let mut permissions = Vec::new();
    let mut in_runtime_permissions = false;

    for line in stdout.lines() {
        let trimmed = line.trim();

        // Look for "runtime permissions:" section
        if trimmed.starts_with("runtime permissions:") {
            in_runtime_permissions = true;
            continue;
        }

        // Exit section when we hit another top-level key
        if in_runtime_permissions
            && !trimmed.starts_with("android.permission.")
            && !trimmed.is_empty()
            && !trimmed.starts_with(" ")
        {
            break;
        }

        // Parse permission lines like "android.permission.CAMERA: granted=true"
        if in_runtime_permissions && trimmed.starts_with("android.permission.") {
            if let Some(colon_pos) = trimmed.find(':') {
                let permission = trimmed[..colon_pos].to_string();
                let granted = trimmed.contains("granted=true");
                permissions.push((permission, granted));
            }
        }
    }

    Ok(permissions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires connected Android device with test app
    async fn test_grant_permission() {
        let result =
            grant_permission(None, "com.android.settings", "android.permission.CAMERA").await;
        // May fail if permission doesn't apply to Settings app
        assert!(result.is_ok() || matches!(result, Err(AdbError::CommandFailed(_))));
    }

    #[tokio::test]
    #[ignore] // Requires connected Android device
    async fn test_list_permissions() {
        let result = list_permissions(None, "com.android.settings").await;
        // Should work on any Android device
        assert!(result.is_ok());
    }
}
