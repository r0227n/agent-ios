//! Android app management via native ADB protocol.
//!
//! This module provides functions for launching, terminating, installing,
//! and managing Android applications using the native ADB protocol.

#![allow(dead_code)]

use super::commands::{AdbError, Result};
use super::connection::AdbConnection;
use serde::{Deserialize, Serialize};

const ACTIVITY_LAUNCH_FAILURE_PATTERNS: &[&str] = &["Error", "Exception"];
const CLEAR_DATA_FAILURE_PATTERNS: &[&str] = &["Failed", "Exception"];
const INSTALL_REPLACE_FAILURE_PATTERNS: &[&str] = &["Failure"];
const REINSTALL_TEMP_APK_PATH: &str = "/data/local/tmp/agent_mobile_install.apk";

/// Installed app information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Android package name.
    pub package_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Human-readable version string, when available.
    pub version_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Internal version code, when available.
    pub version_code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Install location or APK path reported by the device.
    pub path: Option<String>,
}

fn trimmed_output_matches_any(output: &str, patterns: &[&str]) -> bool {
    let trimmed = output.trim();
    !trimmed.is_empty() && patterns.iter().any(|pattern| trimmed.contains(pattern))
}

fn remove_remote_file(conn: &mut AdbConnection, remote_path: &str) {
    let _ = conn.shell_command_args(&["rm", "-f", remote_path]);
}

fn install_replacing_existing(conn: &mut AdbConnection, apk_path: &str) -> Result<()> {
    let file = std::fs::File::open(apk_path).map_err(AdbError::ExecutionError)?;
    conn.push(file, REINSTALL_TEMP_APK_PATH)?;

    let install_output = conn.shell_command_args(&["pm", "install", "-r", REINSTALL_TEMP_APK_PATH]);
    remove_remote_file(conn, REINSTALL_TEMP_APK_PATH);

    let install_output = install_output?;
    if trimmed_output_matches_any(&install_output, INSTALL_REPLACE_FAILURE_PATTERNS) {
        return Err(AdbError::CommandFailed(format!(
            "install failed: {}",
            install_output.trim()
        )));
    }

    Ok(())
}

fn parse_package_list(stdout: &str) -> Vec<AppInfo> {
    stdout
        .lines()
        .filter_map(|line| {
            line.strip_prefix("package:").map(|pkg| AppInfo {
                package_name: pkg.trim().to_string(),
                version_name: None,
                version_code: None,
                path: None,
            })
        })
        .collect()
}

fn parse_package_info(package_name: &str, stdout: &str) -> AppInfo {
    let mut version_name = None;
    let mut version_code = None;
    let mut path = None;

    for line in stdout.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("versionName=") {
            version_name = Some(value.to_string());
            continue;
        }

        if let Some(value) = line.strip_prefix("versionCode=") {
            let code_part = value.split_whitespace().next().unwrap_or("");
            version_code = code_part.parse().ok();
            continue;
        }

        if let Some(value) = line.strip_prefix("codePath=") {
            path = Some(value.to_string());
        }
    }

    AppInfo {
        package_name: package_name.to_string(),
        version_name,
        version_code,
        path,
    }
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
    if trimmed_output_matches_any(&output, ACTIVITY_LAUNCH_FAILURE_PATTERNS) {
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
        // adb_client's install doesn't support -r, so push + install via pm.
        install_replacing_existing(&mut conn, apk_path)?;
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

    Ok(parse_package_list(&stdout))
}

/// Get detailed info about a specific package.
pub async fn get_package_info(serial: Option<&str>, package_name: &str) -> Result<AppInfo> {
    let mut conn = AdbConnection::for_device(serial)?;
    let stdout = conn.shell_command_args(&["dumpsys", "package", package_name])?;
    Ok(parse_package_info(package_name, &stdout))
}

/// Clear app data.
pub async fn clear_data(serial: Option<&str>, package_name: &str) -> Result<()> {
    let mut conn = AdbConnection::for_device(serial)?;
    let output = conn.shell_command_args(&["pm", "clear", package_name])?;
    if trimmed_output_matches_any(&output, CLEAR_DATA_FAILURE_PATTERNS) {
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

    #[test]
    fn test_parse_package_list() {
        let stdout = "package:com.example.one\npackage: com.example.two\nignored";
        let packages = parse_package_list(stdout);

        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].package_name, "com.example.one");
        assert_eq!(packages[1].package_name, "com.example.two");
    }

    #[test]
    fn test_parse_package_info() {
        let stdout = r#"
            versionCode=123 minSdk=24 targetSdk=34
            versionName=1.2.3
            codePath=/data/app/~~abc/base.apk
        "#;

        let info = parse_package_info("com.example.app", stdout);

        assert_eq!(info.package_name, "com.example.app");
        assert_eq!(info.version_name.as_deref(), Some("1.2.3"));
        assert_eq!(info.version_code, Some(123));
        assert_eq!(info.path.as_deref(), Some("/data/app/~~abc/base.apk"));
    }

    #[test]
    fn test_trimmed_output_matches_any() {
        assert!(trimmed_output_matches_any(
            "  Failure [INSTALL_FAILED]  ",
            INSTALL_REPLACE_FAILURE_PATTERNS
        ));
        assert!(!trimmed_output_matches_any(
            "Success",
            INSTALL_REPLACE_FAILURE_PATTERNS
        ));
    }
}
