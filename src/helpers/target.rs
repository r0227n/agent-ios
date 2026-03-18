//! Shared helpers for platform and device target resolution.

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::CommandResult;

/// A fully resolved target device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTarget {
    /// The resolved platform for the target device.
    pub platform: Platform,
    /// The resolved UDID or serial for the target device.
    pub udid: String,
}

/// Detect the platform for an explicitly selected device UDID or serial.
pub async fn detect_platform_from_udid(udid: &str) -> CommandResult<Platform> {
    let mut last_err: Option<String> = None;

    match agent_mobile_platform_ios::simctl::list_simulators() {
        Ok(targets) => {
            if targets.iter().any(|target| target.udid == udid) {
                return Ok(Platform::Ios);
            }
        }
        Err(err) => last_err = Some(format!("Failed to list iOS devices: {err}")),
    }

    if agent_mobile_platform_android::adb::is_adb_available() {
        match agent_mobile_platform_android::adb::list_devices() {
            Ok(devices) => {
                if devices.iter().any(|(serial, _)| serial == udid) {
                    return Ok(Platform::Android);
                }
            }
            Err(err) => last_err = Some(format!("Failed to list Android devices: {err}")),
        }
    }

    if let Some(err) = last_err {
        return Err(format!("Platform detection failed for '{}': {}", udid, err).into());
    }

    Err(format!("Device not found: {}", udid).into())
}

/// Detect a platform from an optional device selector.
pub async fn detect_platform(udid: Option<&str>) -> CommandResult<Platform> {
    match udid {
        Some(udid) => detect_platform_from_udid(udid).await,
        None => Ok(DeviceResolver::detect_platform().await?),
    }
}

/// Resolve the default device identifier for the selected platform.
pub fn resolve_default_udid(platform: Platform) -> CommandResult<String> {
    match platform {
        Platform::Ios => Ok(agent_mobile_platform_ios::simctl::get_booted_simulator()?.udid),
        Platform::Android => {
            let devices = agent_mobile_platform_android::adb::list_devices()?;
            if let Some((serial, _)) = devices.first() {
                Ok(serial.clone())
            } else {
                Err("No Android device connected".into())
            }
        }
    }
}

/// Resolve both platform and device identifier with auto-detection when needed.
pub async fn resolve_target(udid: Option<&str>) -> CommandResult<ResolvedTarget> {
    let platform = detect_platform(udid).await?;
    let udid = match udid {
        Some(udid) => udid.to_string(),
        None => resolve_default_udid(platform)?,
    };

    Ok(ResolvedTarget { platform, udid })
}
