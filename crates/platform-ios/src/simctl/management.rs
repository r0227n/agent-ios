//! Simulator management commands via xcrun simctl.
//!
//! This module provides direct access to simulator lifecycle operations.

#![allow(dead_code)]

use thiserror::Error;

use super::helper::{run_simctl, run_simctl_tolerant};

/// Errors returned by `xcrun simctl` management commands.
#[derive(Debug, Error)]
pub enum SimctlError {
    #[error("simctl command failed: {0}")]
    /// `simctl` completed with a failure message.
    CommandFailed(String),

    #[error("simctl execution error: {0}")]
    /// Launching the `simctl` process failed.
    ExecutionError(#[from] std::io::Error),

    #[error("Invalid output: {0}")]
    /// `simctl` returned output that could not be parsed.
    InvalidOutput(String),
}

/// Convenient result type for simulator management helpers.
pub type Result<T> = std::result::Result<T, SimctlError>;

/// Boot a simulator
pub fn boot(udid: &str) -> Result<()> {
    run_simctl_tolerant(&["boot", udid], &["Booted"])?;
    super::cache::invalidate_cache();
    Ok(())
}

/// Shutdown a simulator
pub fn shutdown(udid: &str) -> Result<()> {
    run_simctl_tolerant(&["shutdown", udid], &["Shutdown"])?;
    super::cache::invalidate_cache();
    Ok(())
}

/// Erase a simulator (reset to clean state)
pub fn erase(udid: &str) -> Result<()> {
    run_simctl(&["erase", udid])?;
    super::cache::invalidate_cache();
    Ok(())
}

/// Create a new simulator.
/// Returns the UDID of the created simulator.
pub fn create(name: &str, device_type: &str, runtime: &str) -> Result<String> {
    let stdout = run_simctl(&["create", name, device_type, runtime])?;
    let udid = stdout.trim().to_string();
    if udid.is_empty() {
        return Err(SimctlError::InvalidOutput(
            "No UDID returned from create command".to_string(),
        ));
    }
    super::cache::invalidate_cache();
    Ok(udid)
}

/// Clone a simulator.
/// Returns the UDID of the cloned simulator.
pub fn clone(udid: &str) -> Result<String> {
    let name = format!("Clone of {}", udid);
    let stdout = run_simctl(&["clone", udid, &name])?;
    let new_udid = stdout.trim().to_string();
    if new_udid.is_empty() {
        return Err(SimctlError::InvalidOutput(
            "No UDID returned from clone command".to_string(),
        ));
    }
    super::cache::invalidate_cache();
    Ok(new_udid)
}

/// Delete a simulator
pub fn delete(udid: &str) -> Result<()> {
    run_simctl(&["delete", udid])?;
    super::cache::invalidate_cache();
    Ok(())
}

/// Delete all simulators
pub fn delete_all() -> Result<()> {
    run_simctl(&["delete", "all"])?;
    super::cache::invalidate_cache();
    Ok(())
}

/// List all simulator devices as JSON string.
pub fn list_devices_json() -> Result<String> {
    run_simctl(&["list", "devices", "--json"])
}

/// Grant privacy permission using simctl.
pub fn privacy_grant(udid: &str, service: &str, bundle_id: &str) -> Result<()> {
    run_simctl(&["privacy", udid, "grant", service, bundle_id])?;
    Ok(())
}

/// Revoke privacy permission using simctl.
pub fn privacy_revoke(udid: &str, service: &str, bundle_id: &str) -> Result<()> {
    run_simctl(&["privacy", udid, "revoke", service, bundle_id])?;
    Ok(())
}

/// Reset privacy permissions for an app.
pub fn privacy_reset(udid: &str, service: &str, bundle_id: &str) -> Result<()> {
    run_simctl(&["privacy", udid, "reset", service, bundle_id])?;
    Ok(())
}

/// Information about a booted simulator.
#[derive(Debug, Clone)]
pub struct BootedSimulator {
    /// Simulator UDID.
    pub udid: String,
    /// Human-readable simulator name.
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
    run_simctl(&["install", udid, path])?;
    Ok(())
}

/// Uninstall an app from a simulator via simctl.
pub fn uninstall_app(udid: &str, bundle_id: &str) -> Result<()> {
    run_simctl(&["uninstall", udid, bundle_id])?;
    Ok(())
}

/// Simplified app info from simctl listapps.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimctlAppInfo {
    /// Application bundle identifier.
    pub bundle_id: String,
    /// Display name reported by the application bundle.
    pub name: String,
    /// Marketing version string.
    pub version: String,
    /// Application classification such as `User` or `System`.
    pub app_type: String,
}

/// List installed apps on a simulator via simctl.
///
/// Runs `xcrun simctl listapps` (plist output) and parses directly
/// with the `plist` crate to get structured data.
pub fn list_apps(udid: &str) -> Result<Vec<SimctlAppInfo>> {
    let stdout = run_simctl(&["listapps", udid])?;
    let plist_value: plist::Value = plist::from_bytes(stdout.as_bytes())
        .map_err(|e| SimctlError::InvalidOutput(format!("plist parse error: {}", e)))?;

    let dict = plist_value
        .as_dictionary()
        .ok_or_else(|| SimctlError::InvalidOutput("Expected plist dictionary".to_string()))?;

    let mut apps = Vec::new();
    for (bundle_id, info) in dict {
        let info_dict = match info.as_dictionary() {
            Some(d) => d,
            None => continue,
        };

        let name = info_dict
            .get("CFBundleDisplayName")
            .or_else(|| info_dict.get("CFBundleName"))
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
        let version = info_dict
            .get("CFBundleShortVersionString")
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();
        let app_type = info_dict
            .get("ApplicationType")
            .and_then(|v| v.as_string())
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
/// Results are cached with a 5-second TTL to avoid repeated `simctl` invocations.
/// The cache is automatically invalidated by lifecycle operations (boot, shutdown, etc.).
///
/// Runs `xcrun simctl list devices --json` and converts each device
/// into a `DeviceInfo` with `target_type` set to `Simulator`.
///
/// The OS version is extracted from the runtime identifier
/// (e.g., `com.apple.CoreSimulator.SimRuntime.iOS-17-0` → `iOS 17.0`).
pub fn list_simulators() -> Result<Vec<agent_mobile_core::types::DeviceInfo>> {
    use super::cache;
    use agent_mobile_core::types::{DeviceInfo, TargetType};

    // Check cache first
    if let Some(cached) = cache::get_cached_devices() {
        return Ok(cached);
    }

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

    // Store in cache
    cache::cache_devices(result.clone());

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
