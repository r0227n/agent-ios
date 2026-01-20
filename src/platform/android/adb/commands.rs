//! ADB command execution module.
//!
//! This module provides functions to execute ADB commands and parse their output
//! for Android device/emulator management.

use std::process::Command;
use thiserror::Error;

/// ADB command execution errors.
#[derive(Debug, Error)]
pub enum AdbError {
    #[error("adb command failed: {0}")]
    CommandFailed(String),

    #[error("adb execution error: {0}")]
    ExecutionError(#[from] std::io::Error),

    #[error("adb not found in PATH. Please install Android SDK platform-tools.")]
    AdbNotFound,

    #[error("Invalid output: {0}")]
    InvalidOutput(String),
}

pub type Result<T> = std::result::Result<T, AdbError>;

/// Check if adb is available in PATH.
pub fn is_adb_available() -> bool {
    Command::new("adb").arg("version").output().is_ok()
}

/// List connected devices via `adb devices`.
/// Returns a vector of (serial, state) tuples.
pub fn list_devices() -> Result<Vec<(String, String)>> {
    let output = Command::new("adb")
        .args(["devices"])
        .output()
        .map_err(|e| {
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
    let mut devices = Vec::new();

    for line in stdout.lines().skip(1) {
        // Skip "List of devices attached" header
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            devices.push((parts[0].to_string(), parts[1].to_string()));
        }
    }

    Ok(devices)
}

/// List available AVDs via `emulator -list-avds`.
pub fn list_avds() -> Result<Vec<String>> {
    let output = Command::new("emulator")
        .args(["-list-avds"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AdbError::CommandFailed("emulator command not found".to_string())
            } else {
                AdbError::ExecutionError(e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let avds: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(avds)
}

/// Get a device property via `adb shell getprop`.
fn get_prop(serial: &str, prop: &str) -> Result<String> {
    let output = Command::new("adb")
        .args(["-s", serial, "shell", "getprop", prop])
        .output()
        .map_err(|e| {
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

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get API level (SDK version) for a device.
pub fn get_api_level(serial: &str) -> Result<Option<u32>> {
    let value = get_prop(serial, "ro.build.version.sdk")?;
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
    let value = get_prop(serial, "ro.build.version.release")?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Get device model name.
pub fn get_device_model(serial: &str) -> Result<Option<String>> {
    let value = get_prop(serial, "ro.product.model")?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Get AVD name for an emulator.
pub fn get_avd_name(serial: &str) -> Result<Option<String>> {
    let value = get_prop(serial, "ro.boot.qemu.avd_name")?;
    if value.is_empty() {
        Ok(None)
    } else {
        // AVD name may be URL-encoded, decode underscores
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
        assert!(err.to_string().contains("adb not found"));
    }

    #[test]
    fn test_is_emulator() {
        assert!(is_emulator("emulator-5554"));
        assert!(is_emulator("emulator-5556"));
        assert!(!is_emulator("12345678"));
        assert!(!is_emulator("device-serial"));
    }
}
