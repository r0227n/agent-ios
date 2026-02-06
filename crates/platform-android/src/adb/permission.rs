//! Permission management module for Android apps.
//!
//! This module provides functions to grant, revoke, and reset runtime permissions
//! for Android apps using the native ADB protocol.

use super::commands::{AdbError, Result};
use super::connection::AdbConnection;

/// Grant a runtime permission to an app.
///
/// Uses `pm grant <package> <permission>` via native ADB protocol.
pub async fn grant_permission(serial: Option<&str>, package: &str, permission: &str) -> Result<()> {
    let serial = serial.map(|s| s.to_string());
    let package = package.to_string();
    let permission = permission.to_string();

    tokio::task::spawn_blocking(move || {
        let mut conn = AdbConnection::for_device(serial.as_deref())?;
        let output = conn.shell_command_args(&["pm", "grant", &package, &permission])?;

        // pm grant outputs error messages on failure (successful grants produce empty output)
        let trimmed = output.trim();
        if !trimmed.is_empty()
            && (trimmed.contains("Exception")
                || trimmed.contains("Error")
                || trimmed.contains("Unknown"))
        {
            return Err(AdbError::CommandFailed(format!(
                "Failed to grant permission '{}' to '{}': {}",
                permission, package, trimmed
            )));
        }

        Ok(())
    })
    .await
    .map_err(|e| AdbError::CommandFailed(format!("task join error: {}", e)))?
}

/// Revoke a runtime permission from an app.
///
/// Uses `pm revoke <package> <permission>` via native ADB protocol.
pub async fn revoke_permission(
    serial: Option<&str>,
    package: &str,
    permission: &str,
) -> Result<()> {
    let serial = serial.map(|s| s.to_string());
    let package = package.to_string();
    let permission = permission.to_string();

    tokio::task::spawn_blocking(move || {
        let mut conn = AdbConnection::for_device(serial.as_deref())?;
        let output = conn.shell_command_args(&["pm", "revoke", &package, &permission])?;

        // pm revoke outputs error messages on failure (successful revokes produce empty output)
        let trimmed = output.trim();
        if !trimmed.is_empty()
            && (trimmed.contains("Exception")
                || trimmed.contains("Error")
                || trimmed.contains("Unknown"))
        {
            return Err(AdbError::CommandFailed(format!(
                "Failed to revoke permission '{}' from '{}': {}",
                permission, package, trimmed
            )));
        }

        Ok(())
    })
    .await
    .map_err(|e| AdbError::CommandFailed(format!("task join error: {}", e)))?
}

/// Reset all permissions for an app to their default state.
///
/// Uses `pm reset-permissions <package>` via native ADB protocol.
///
/// # Note
/// This requires Android 6.0 (API 23) or higher.
pub async fn reset_permissions(serial: Option<&str>, package: &str) -> Result<()> {
    let serial = serial.map(|s| s.to_string());
    let package = package.to_string();

    tokio::task::spawn_blocking(move || {
        let mut conn = AdbConnection::for_device(serial.as_deref())?;
        let output = conn.shell_command_args(&["pm", "reset-permissions", &package])?;

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
    })
    .await
    .map_err(|e| AdbError::CommandFailed(format!("task join error: {}", e)))?
}

/// List all runtime permissions for a package.
///
/// Uses `dumpsys package <package>` via native ADB protocol and parses the permissions section.
pub async fn list_permissions(serial: Option<&str>, package: &str) -> Result<Vec<(String, bool)>> {
    let serial = serial.map(|s| s.to_string());
    let package = package.to_string();

    let stdout = tokio::task::spawn_blocking(move || {
        let mut conn = AdbConnection::for_device(serial.as_deref())?;
        conn.shell_command_args(&["dumpsys", "package", &package])
    })
    .await
    .map_err(|e| AdbError::CommandFailed(format!("task join error: {}", e)))??;

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
            && !line.starts_with(" ")
            && !line.starts_with("\t")
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
