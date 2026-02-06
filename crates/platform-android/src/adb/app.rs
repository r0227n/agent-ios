//! Android app management via native ADB protocol.
//!
//! This module provides functions for launching, terminating, installing,
//! and managing Android applications using the native ADB protocol.

#![allow(dead_code)]

use super::commands::{AdbError, Result};
use super::connection::AdbConnection;
use serde::{Deserialize, Serialize};

/// Installed app information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub package_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Launch an app by package name.
///
/// Uses `monkey` to launch the app's main LAUNCHER activity.
pub async fn launch(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    let output = conn.shell_command_args(&[
        "monkey",
        "-p",
        package_name,
        "-c",
        "android.intent.category.LAUNCHER",
        "1",
    ])?;

    // monkey returns success even on some failures, check output
    if output.contains("No activities found") {
        return Err(AdbError::CommandFailed(format!(
            "No LAUNCHER activity found for {}",
            package_name
        )));
    }

    Ok(())
}

/// Launch an app with specific activity.
pub async fn launch_activity(
    serial: Option<&str>,
    package_name: &str,
    activity: &str,
) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    let component = format!("{}/{}", package_name, activity);
    let output = conn.shell_command_args(&["am", "start", "-n", &component])?;
    if output.contains("Error") || output.contains("Exception") {
        return Err(AdbError::CommandFailed(format!(
            "Failed to launch activity '{}': {}",
            component,
            output.trim()
        )));
    }
    Ok(())
}

/// Terminate (force stop) an app.
pub async fn terminate(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    conn.shell_command_args(&["am", "force-stop", package_name])?;
    Ok(())
}

/// Install an APK.
pub async fn install(serial: Option<&str>, apk_path: &str, reinstall: bool) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;

    if reinstall {
        // For reinstall, use shell pm install with -r flag via push + pm install
        // adb_client's install doesn't support -r flag, so use shell command approach
        let temp_path = "/data/local/tmp/agent_mobile_install.apk";

        // Push APK to device
        let file = std::fs::File::open(apk_path).map_err(AdbError::ExecutionError)?;
        conn.push(file, temp_path)?;

        // Install from device temp path with -r flag
        let output = conn.shell_command_args(&["pm", "install", "-r", temp_path])?;
        if output.contains("Failure") {
            // Cleanup
            let _ = conn.shell_command_args(&["rm", "-f", temp_path]);
            return Err(AdbError::CommandFailed(format!(
                "install failed: {}",
                output.trim()
            )));
        }

        // Cleanup
        let _ = conn.shell_command_args(&["rm", "-f", temp_path]);
    } else {
        conn.install(apk_path)?;
    }

    Ok(())
}

/// Uninstall an app.
pub async fn uninstall(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    conn.uninstall(package_name)?;
    Ok(())
}

/// List installed packages.
pub async fn list_packages(serial: Option<&str>, third_party_only: bool) -> Result<Vec<AppInfo>> {
    let mut conn = AdbConnection::for_device(serial)?;

    let stdout = if third_party_only {
        conn.shell_command_args(&["pm", "list", "packages", "-3"])?
    } else {
        conn.shell_command_args(&["pm", "list", "packages"])?
    };

    let apps: Vec<AppInfo> = stdout
        .lines()
        .filter_map(|line| {
            line.strip_prefix("package:").map(|pkg| AppInfo {
                package_name: pkg.trim().to_string(),
                version_name: None,
                version_code: None,
                path: None,
            })
        })
        .collect();

    Ok(apps)
}

/// Get detailed info about a specific package.
pub async fn get_package_info(serial: Option<&str>, package_name: &str) -> Result<AppInfo> {
    let mut conn = AdbConnection::for_device(serial)?;
    let stdout = conn.shell_command_args(&["dumpsys", "package", package_name])?;

    let mut version_name = None;
    let mut version_code = None;
    let mut path = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("versionName=") {
            version_name = line.strip_prefix("versionName=").map(|s| s.to_string());
        } else if line.starts_with("versionCode=") {
            if let Some(code_str) = line.strip_prefix("versionCode=") {
                // Format might be "versionCode=123 minSdk=..."
                let code_part = code_str.split_whitespace().next().unwrap_or("");
                version_code = code_part.parse().ok();
            }
        } else if line.starts_with("codePath=") {
            path = line.strip_prefix("codePath=").map(|s| s.to_string());
        }
    }

    Ok(AppInfo {
        package_name: package_name.to_string(),
        version_name,
        version_code,
        path,
    })
}

/// Clear app data.
pub async fn clear_data(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    let output = conn.shell_command_args(&["pm", "clear", package_name])?;
    if output.contains("Failed") || output.contains("Exception") {
        return Err(AdbError::CommandFailed(format!(
            "Failed to clear data for '{}': {}",
            package_name,
            output.trim()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_info() {
        let info = AppInfo {
            package_name: "com.example.app".to_string(),
            version_name: Some("1.0.0".to_string()),
            version_code: Some(1),
            path: None,
        };
        assert_eq!(info.package_name, "com.example.app");
    }
}
