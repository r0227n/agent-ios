//! Simulator management commands via xcrun simctl.
//!
//! This module provides direct access to simulator lifecycle operations
//! without going through idb_companion.

#![allow(dead_code)]

use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimctlError {
    #[error("simctl command failed: {0}")]
    CommandFailed(String),

    #[error("simctl execution error: {0}")]
    ExecutionError(#[from] std::io::Error),

    #[error("Invalid output: {0}")]
    InvalidOutput(String),
}

pub type Result<T> = std::result::Result<T, SimctlError>;

/// Image format for screenshots
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Gif,
    Tiff,
}

impl ImageFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Bmp => "bmp",
            Self::Gif => "gif",
            Self::Tiff => "tiff",
        }
    }

    /// Get the simctl type parameter for this format
    pub fn simctl_type(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Bmp => "bmp",
            Self::Gif => "gif",
            Self::Tiff => "tiff",
        }
    }
}

/// Boot a simulator
pub fn boot(udid: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "boot", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "Unable to boot device in current state: Booted" is not an error
        if stderr.contains("Booted") {
            return Ok(());
        }
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Shutdown a simulator
pub fn shutdown(udid: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "shutdown", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "Unable to shutdown device in current state: Shutdown" is not an error
        if stderr.contains("Shutdown") {
            return Ok(());
        }
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Erase a simulator (reset to clean state)
pub fn erase(udid: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "erase", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Create a new simulator.
/// Returns the UDID of the created simulator.
pub fn create(name: &str, device_type: &str, runtime: &str) -> Result<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "create", name, device_type, runtime])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    let udid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if udid.is_empty() {
        return Err(SimctlError::InvalidOutput(
            "No UDID returned from create command".to_string(),
        ));
    }

    Ok(udid)
}

/// Clone a simulator.
/// Returns the UDID of the cloned simulator.
pub fn clone(udid: &str) -> Result<String> {
    let name = format!("Clone of {}", udid);

    let output = Command::new("xcrun")
        .args(["simctl", "clone", udid, &name])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    let new_udid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if new_udid.is_empty() {
        return Err(SimctlError::InvalidOutput(
            "No UDID returned from clone command".to_string(),
        ));
    }

    Ok(new_udid)
}

/// Delete a simulator
pub fn delete(udid: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "delete", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Delete all simulators
pub fn delete_all() -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "delete", "all"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// List all simulator devices as JSON string.
pub fn list_devices_json() -> Result<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "--json"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    String::from_utf8(output.stdout).map_err(|e| SimctlError::InvalidOutput(e.to_string()))
}

/// Copy text to simulator clipboard.
pub fn pbcopy(udid: &str, text: &str) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("xcrun")
        .args(["simctl", "pbcopy", udid])
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(SimctlError::CommandFailed("pbcopy failed".to_string()));
    }

    Ok(())
}

/// Paste text from simulator clipboard.
pub fn pbpaste(udid: &str) -> Result<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "pbpaste", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Grant privacy permission using simctl.
pub fn privacy_grant(udid: &str, service: &str, bundle_id: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "privacy", udid, "grant", service, bundle_id])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Revoke privacy permission using simctl.
pub fn privacy_revoke(udid: &str, service: &str, bundle_id: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "privacy", udid, "revoke", service, bundle_id])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Reset privacy permissions for an app.
pub fn privacy_reset(udid: &str, service: &str, bundle_id: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "privacy", udid, "reset", service, bundle_id])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Take a screenshot of a simulator using xcrun simctl io with specified format.
pub fn io_screenshot(udid: &str, output_path: &str, format: ImageFormat) -> Result<()> {
    let output = Command::new("xcrun")
        .args([
            "simctl",
            "io",
            udid,
            "screenshot",
            &format!("--type={}", format.simctl_type()),
            output_path,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Take a screenshot and return bytes with specified format.
pub fn io_screenshot_bytes(udid: &str, format: ImageFormat) -> Result<Vec<u8>> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_path = format!(
        "/tmp/agent_mobile_screenshot_{}_{}.{}",
        std::process::id(),
        nanos,
        format.extension()
    );

    io_screenshot(udid, &temp_path, format)?;

    let data = std::fs::read(&temp_path);
    let _ = std::fs::remove_file(&temp_path); // Cleanup regardless of read success

    let data =
        data.map_err(|e| SimctlError::InvalidOutput(format!("Failed to read screenshot: {}", e)))?;

    Ok(data)
}

/// Information about a booted simulator.
#[derive(Debug, Clone)]
pub struct BootedSimulator {
    pub udid: String,
    pub name: String,
}

/// Find the first booted simulator.
///
/// Parses `xcrun simctl list devices --json` output and returns the
/// first device whose state is "Booted".
pub fn get_booted_simulator() -> Result<BootedSimulator> {
    let json_str = list_devices_json()?;
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| SimctlError::InvalidOutput(e.to_string()))?;

    let devices = parsed
        .get("devices")
        .and_then(|d| d.as_object())
        .ok_or_else(|| SimctlError::InvalidOutput("Missing 'devices' key".to_string()))?;

    for (_runtime, device_list) in devices {
        if let Some(arr) = device_list.as_array() {
            for device in arr {
                let state = device.get("state").and_then(|s| s.as_str());
                if state == Some("Booted") {
                    let udid = device
                        .get("udid")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = device
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !udid.is_empty() {
                        return Ok(BootedSimulator { udid, name });
                    }
                }
            }
        }
    }

    Err(SimctlError::CommandFailed(
        "No booted simulator found".to_string(),
    ))
}

/// Install an app on a simulator via simctl.
pub fn install_app(udid: &str, path: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "install", udid, path])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Uninstall an app from a simulator via simctl.
pub fn uninstall_app(udid: &str, bundle_id: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "uninstall", udid, bundle_id])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SimctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

/// Simplified app info from simctl listapps.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimctlAppInfo {
    pub bundle_id: String,
    pub name: String,
    pub version: String,
    pub app_type: String,
}

/// List installed apps on a simulator via simctl.
///
/// Runs `xcrun simctl listapps` (plist output) and pipes through
/// `plutil -convert json` to get JSON, then parses into structured data.
pub fn list_apps(udid: &str) -> Result<Vec<SimctlAppInfo>> {
    use std::process::Stdio;

    // Run simctl listapps and pipe through plutil for JSON conversion
    let simctl = Command::new("xcrun")
        .args(["simctl", "listapps", udid])
        .stdout(Stdio::piped())
        .spawn()?;

    let plutil = Command::new("plutil")
        .args(["-convert", "json", "-o", "-", "--", "-"])
        .stdin(simctl.stdout.ok_or_else(|| {
            SimctlError::CommandFailed("Failed to capture simctl stdout".to_string())
        })?)
        .output()?;

    if !plutil.status.success() {
        let stderr = String::from_utf8_lossy(&plutil.stderr);
        return Err(SimctlError::CommandFailed(format!(
            "plutil conversion failed: {}",
            stderr
        )));
    }

    let json_str =
        String::from_utf8(plutil.stdout).map_err(|e| SimctlError::InvalidOutput(e.to_string()))?;

    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| SimctlError::InvalidOutput(e.to_string()))?;

    let obj = parsed
        .as_object()
        .ok_or_else(|| SimctlError::InvalidOutput("Expected JSON object".to_string()))?;

    let mut apps = Vec::new();
    for (bundle_id, info) in obj {
        let name = info
            .get("CFBundleDisplayName")
            .or_else(|| info.get("CFBundleName"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let version = info
            .get("CFBundleShortVersionString")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let app_type = info
            .get("ApplicationType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        apps.push(SimctlAppInfo {
            bundle_id: bundle_id.clone(),
            name,
            version,
            app_type,
        });
    }

    // Sort by bundle_id for consistent output
    apps.sort_by(|a, b| a.bundle_id.cmp(&b.bundle_id));
    Ok(apps)
}

/// List all simulators as DeviceInfo structs.
///
/// Runs `xcrun simctl list devices --json` and converts each device
/// into a `DeviceInfo` with `target_type` set to `Simulator`.
///
/// The OS version is extracted from the runtime identifier
/// (e.g., `com.apple.CoreSimulator.SimRuntime.iOS-17-0` → `iOS 17.0`).
pub fn list_simulators() -> Result<Vec<agent_mobile_core::types::DeviceInfo>> {
    use agent_mobile_core::types::{DeviceInfo, TargetType};

    let json_str = list_devices_json()?;
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| SimctlError::InvalidOutput(e.to_string()))?;

    let devices = parsed
        .get("devices")
        .and_then(|d| d.as_object())
        .ok_or_else(|| SimctlError::InvalidOutput("Missing 'devices' key".to_string()))?;

    let mut result = Vec::new();

    for (runtime, device_list) in devices {
        let os_version = parse_runtime_version(runtime);

        if let Some(arr) = device_list.as_array() {
            for device in arr {
                let udid = device
                    .get("udid")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                if udid.is_empty() {
                    continue;
                }

                let name = device
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let state = device
                    .get("state")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());

                result.push(DeviceInfo {
                    name,
                    udid,
                    state,
                    target_type: TargetType::Simulator,
                    os_version: os_version.clone(),
                    architecture: None,
                    companion_info: None,
                });
            }
        }
    }

    Ok(result)
}

/// Extract OS version from a simctl runtime identifier.
///
/// e.g. `com.apple.CoreSimulator.SimRuntime.iOS-17-0` → `Some("iOS 17.0")`
fn parse_runtime_version(runtime: &str) -> Option<String> {
    // Runtime format: com.apple.CoreSimulator.SimRuntime.<OS>-<Major>-<Minor>
    let prefix = "com.apple.CoreSimulator.SimRuntime.";
    let suffix = runtime.strip_prefix(prefix)?;
    // suffix is like "iOS-17-0" or "tvOS-17-0"
    let version = suffix.replace('-', " ");
    // Insert dot between major and minor: "iOS 17 0" → "iOS 17.0"
    // Split into parts and rejoin
    let parts: Vec<&str> = version.split(' ').collect();
    if parts.len() >= 3 {
        // e.g. ["iOS", "17", "0"] → "iOS 17.0"
        let os_name = parts[0];
        let version_parts = &parts[1..];
        Some(format!("{} {}", os_name, version_parts.join(".")))
    } else if parts.len() == 2 {
        Some(version)
    } else {
        Some(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simctl_error_display() {
        let err = SimctlError::CommandFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_parse_runtime_version() {
        assert_eq!(
            parse_runtime_version("com.apple.CoreSimulator.SimRuntime.iOS-17-0"),
            Some("iOS 17.0".to_string())
        );
        assert_eq!(
            parse_runtime_version("com.apple.CoreSimulator.SimRuntime.tvOS-18-2"),
            Some("tvOS 18.2".to_string())
        );
        assert_eq!(parse_runtime_version("unknown-runtime"), None);
    }
}
