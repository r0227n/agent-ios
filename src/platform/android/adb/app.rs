//! Android app management via adb am/pm commands.
//!
//! This module provides functions for launching, terminating, installing,
//! and managing Android applications.

#![allow(dead_code)]

use super::{AdbError, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

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
/// Uses `adb shell am start` to launch the app's main activity.
pub async fn launch(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    // Use monkey to launch (simpler than finding the main activity)
    cmd.args([
        "shell",
        "monkey",
        "-p",
        package_name,
        "-c",
        "android.intent.category.LAUNCHER",
        "1",
    ]);

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AdbError::CommandFailed(format!(
            "stdout: {}, stderr: {}",
            stdout, stderr
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
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    let component = format!("{}/{}", package_name, activity);
    cmd.args(["shell", "am", "start", "-n", &component]);

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Terminate (force stop) an app.
pub async fn terminate(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.args(["shell", "am", "force-stop", package_name]);

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Install an APK.
pub async fn install(serial: Option<&str>, apk_path: &str, reinstall: bool) -> Result<()> {
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.arg("install");
    if reinstall {
        cmd.arg("-r");
    }
    cmd.arg(apk_path);

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || stdout.contains("Failure") {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(format!(
            "stdout: {}, stderr: {}",
            stdout, stderr
        )));
    }

    Ok(())
}

/// Uninstall an app.
pub async fn uninstall(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.args(["uninstall", package_name]);

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || stdout.contains("Failure") {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(format!(
            "stdout: {}, stderr: {}",
            stdout, stderr
        )));
    }

    Ok(())
}

/// List installed packages.
pub async fn list_packages(serial: Option<&str>, third_party_only: bool) -> Result<Vec<AppInfo>> {
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.args(["shell", "pm", "list", "packages"]);
    if third_party_only {
        cmd.arg("-3");
    }

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
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
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.args(["shell", "dumpsys", "package", package_name]);

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

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
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.args(["shell", "pm", "clear", package_name]);

    let output = cmd.output().await.map_err(|e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(stderr.to_string()));
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
