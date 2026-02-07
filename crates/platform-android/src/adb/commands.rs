//! ADB command execution module.
//!
//! This module provides functions to manage Android devices/emulators.
//! All operations use the native ADB protocol via `adb_client` crate
//! (TCP :5037 direct communication) instead of shelling out to the `adb` CLI.

#![allow(dead_code)]

use thiserror::Error;

/// ADB command execution errors.
#[derive(Debug, Error)]
pub enum AdbError {
    #[error("adb command failed: {0}")]
    CommandFailed(String),

    #[error("adb execution error: {0}")]
    ExecutionError(#[from] std::io::Error),

    #[error("adb server not reachable at 127.0.0.1:5037. Please start the ADB server with 'adb start-server'.")]
    AdbNotFound,

    #[error("Invalid output: {0}")]
    InvalidOutput(String),
}

pub type Result<T> = std::result::Result<T, AdbError>;

/// Check if ADB server is reachable (native TCP check).
pub fn is_adb_available() -> bool {
    super::connection::is_adb_available()
}

/// List connected devices via ADB protocol.
/// Returns a vector of (serial, state) tuples.
pub fn list_devices() -> Result<Vec<(String, String)>> {
    super::connection::list_devices()
}

/// List available AVDs by reading `~/.android/avd/*.ini` files.
pub fn list_avds() -> Result<Vec<String>> {
    super::connection::list_avds()
}

/// Get API level (SDK version) for a device.
pub fn get_api_level(serial: &str) -> Result<Option<u32>> {
    let value = super::connection::get_prop(serial, "ro.build.version.sdk")?;
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse()
        .map(Some)
        .map_err(|_| AdbError::InvalidOutput(format!("Invalid API level: {}", value)))
}

/// Get Android version string (e.g., "14", "13").
pub fn get_android_version(serial: &str) -> Result<Option<String>> {
    let value = super::connection::get_prop(serial, "ro.build.version.release")?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Get device model name.
pub fn get_device_model(serial: &str) -> Result<Option<String>> {
    let value = super::connection::get_prop(serial, "ro.product.model")?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Get AVD name for an emulator.
pub fn get_avd_name(serial: &str) -> Result<Option<String>> {
    let value = super::connection::get_prop(serial, "ro.boot.qemu.avd_name")?;
    if value.is_empty() {
        Ok(None)
    } else {
        // AVD name uses underscores as space separators (e.g., "Pixel_6_API_34")
        Ok(Some(value.replace("_", " ")))
    }
}

/// Check if device is an emulator.
pub fn is_emulator(serial: &str) -> bool {
    // Emulators typically have serials like "emulator-5554"
    serial.starts_with("emulator-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_error_display() {
        let err = AdbError::CommandFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = AdbError::AdbNotFound;
        assert!(err.to_string().contains("adb server not reachable"));
    }

    #[test]
    fn test_is_emulator() {
        assert!(is_emulator("emulator-5554"));
        assert!(is_emulator("emulator-5556"));
        assert!(!is_emulator("12345678"));
        assert!(!is_emulator("device-serial"));
    }
}
