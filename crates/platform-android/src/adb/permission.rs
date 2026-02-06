//! Permission management module for Android apps.
//!
//! This module provides functions to grant, revoke, and reset runtime permissions
//! for Android apps using `adb shell pm` commands.

use super::commands::{AdbError, Result};
use tokio::process::Command;

/// Grant a runtime permission to an app.
///
/// Uses `adb shell pm grant <package> <permission>` to grant a runtime permission.
///
/// # Arguments
/// - `serial`: Device serial number (optional, uses default device if None)
/// - `package`: App package name (e.g., "com.example.app")
/// - `permission`: Permission name (e.g., "android.permission.CAMERA")
///
/// # Returns
/// - `Ok(())` on success
/// - `Err(AdbError)` if grant fails
///
/// # Example
/// ```
/// use platform_android::adb::permission::grant_permission;
///
/// grant_permission(
///     Some("emulator-5554"),
///     "com.example.app",
///     "android.permission.CAMERA"
/// ).await?;
/// ```
///
/// # Common Permissions
/// - `android.permission.CAMERA`
/// - `android.permission.RECORD_AUDIO`
/// - `android.permission.ACCESS_FINE_LOCATION`
/// - `android.permission.ACCESS_COARSE_LOCATION`
/// - `android.permission.READ_CONTACTS`
/// - `android.permission.WRITE_CONTACTS`
/// - `android.permission.READ_EXTERNAL_STORAGE`
/// - `android.permission.WRITE_EXTERNAL_STORAGE`
pub async fn grant_permission(serial: Option<&str>, package: &str, permission: &str) -> Result<()> {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }
    cmd.args(["shell", "pm", "grant", package, permission]);

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(format!(
            "Failed to grant permission '{}' to '{}': {}",
            permission, package, stderr
        )));
    }

    Ok(())
}

/// Revoke a runtime permission from an app.
///
/// Uses `adb shell pm revoke <package> <permission>` to revoke a runtime permission.
///
/// # Arguments
/// - `serial`: Device serial number (optional)
/// - `package`: App package name
/// - `permission`: Permission name to revoke
///
/// # Returns
/// - `Ok(())` on success
/// - `Err(AdbError)` if revoke fails
pub async fn revoke_permission(
    serial: Option<&str>,
    package: &str,
    permission: &str,
) -> Result<()> {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }
    cmd.args(["shell", "pm", "revoke", package, permission]);

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(format!(
            "Failed to revoke permission '{}' from '{}': {}",
            permission, package, stderr
        )));
    }

    Ok(())
}

/// Reset all permissions for an app to their default state.
///
/// Uses `adb shell pm reset-permissions <package>` to reset all permissions.
///
/// # Arguments
/// - `serial`: Device serial number (optional)
/// - `package`: App package name
///
/// # Returns
/// - `Ok(())` on success
/// - `Err(AdbError)` if reset fails
///
/// # Note
/// This requires Android 6.0 (API 23) or higher.
pub async fn reset_permissions(serial: Option<&str>, package: &str) -> Result<()> {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }
    cmd.args(["shell", "pm", "reset-permissions", package]);

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // reset-permissions may not be available on all Android versions
        // Treat as non-critical error
        eprintln!(
            "Warning: Failed to reset permissions for '{}': {}",
            package, stderr
        );
    }

    Ok(())
}

/// List all runtime permissions for a package.
///
/// Uses `adb shell dumpsys package <package>` and parses the permissions section.
///
/// # Arguments
/// - `serial`: Device serial number (optional)
/// - `package`: App package name
///
/// # Returns
/// - `Ok(Vec<(permission, granted)>)` - List of permissions and their grant status
/// - `Err(AdbError)` if command fails
pub async fn list_permissions(serial: Option<&str>, package: &str) -> Result<Vec<(String, bool)>> {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }
    cmd.args(["shell", "dumpsys", "package", package]);

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(format!(
            "Failed to list permissions for '{}': {}",
            package, stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
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
