//! ADB input command module.
//!
//! This module provides functions to execute ADB input commands for
//! touch/gesture interactions with Android devices.

use super::{AdbError, Result};
use tokio::process::Command;

/// Default swipe duration in milliseconds.
const DEFAULT_SWIPE_DURATION_MS: u64 = 300;

/// Execute an adb shell input command.
async fn adb_input(serial: Option<&str>, args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.arg("shell").arg("input");
    cmd.args(args);

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

/// Tap at the specified screen coordinates.
///
/// # Arguments
/// * `serial` - Optional device serial number
/// * `x` - X coordinate
/// * `y` - Y coordinate
pub async fn tap(serial: Option<&str>, x: f64, y: f64) -> Result<()> {
    let x_str = x.round().to_string();
    let y_str = y.round().to_string();
    adb_input(serial, &["tap", &x_str, &y_str]).await
}

/// Long press at the specified screen coordinates.
///
/// # Arguments
/// * `serial` - Optional device serial number
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `duration_ms` - Press duration in milliseconds
pub async fn long_press(serial: Option<&str>, x: f64, y: f64, duration_ms: u64) -> Result<()> {
    // Android implements long press as a swipe from point to same point with duration
    let x_str = x.round().to_string();
    let y_str = y.round().to_string();
    let duration_str = duration_ms.to_string();
    adb_input(
        serial,
        &["swipe", &x_str, &y_str, &x_str, &y_str, &duration_str],
    )
    .await
}

/// Swipe from one point to another.
///
/// # Arguments
/// * `serial` - Optional device serial number
/// * `x1` - Start X coordinate
/// * `y1` - Start Y coordinate
/// * `x2` - End X coordinate
/// * `y2` - End Y coordinate
/// * `duration_ms` - Optional swipe duration in milliseconds
pub async fn swipe(
    serial: Option<&str>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    duration_ms: Option<u64>,
) -> Result<()> {
    let x1_str = x1.round().to_string();
    let y1_str = y1.round().to_string();
    let x2_str = x2.round().to_string();
    let y2_str = y2.round().to_string();

    match duration_ms {
        Some(d) => {
            let duration_str = d.to_string();
            adb_input(
                serial,
                &["swipe", &x1_str, &y1_str, &x2_str, &y2_str, &duration_str],
            )
            .await
        }
        None => {
            let duration_str = DEFAULT_SWIPE_DURATION_MS.to_string();
            adb_input(
                serial,
                &["swipe", &x1_str, &y1_str, &x2_str, &y2_str, &duration_str],
            )
            .await
        }
    }
}

/// Input text on the device.
///
/// Note: Special characters may need escaping for shell.
///
/// # Arguments
/// * `serial` - Optional device serial number
/// * `text` - Text to input
pub async fn text(serial: Option<&str>, text: &str) -> Result<()> {
    // Escape special shell characters
    let escaped = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace(' ', "%s")
        .replace('&', "\\&")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('|', "\\|")
        .replace(';', "\\;")
        .replace('$', "\\$")
        .replace('`', "\\`");

    adb_input(serial, &["text", &escaped]).await
}

/// Press a key by keycode.
///
/// # Arguments
/// * `serial` - Optional device serial number
/// * `keycode` - Android keycode (e.g., 3 for HOME, 4 for BACK)
pub async fn keyevent(serial: Option<&str>, keycode: u32) -> Result<()> {
    let keycode_str = keycode.to_string();
    adb_input(serial, &["keyevent", &keycode_str]).await
}

/// Press a key by name.
///
/// # Arguments
/// * `serial` - Optional device serial number
/// * `keyname` - Key name (e.g., "KEYCODE_HOME", "KEYCODE_BACK")
pub async fn keyevent_by_name(serial: Option<&str>, keyname: &str) -> Result<()> {
    adb_input(serial, &["keyevent", keyname]).await
}

/// Common Android key codes.
pub mod keycodes {
    pub const HOME: u32 = 3;
    pub const BACK: u32 = 4;
    pub const CALL: u32 = 5;
    pub const ENDCALL: u32 = 6;
    pub const DPAD_UP: u32 = 19;
    pub const DPAD_DOWN: u32 = 20;
    pub const DPAD_LEFT: u32 = 21;
    pub const DPAD_RIGHT: u32 = 22;
    pub const DPAD_CENTER: u32 = 23;
    pub const VOLUME_UP: u32 = 24;
    pub const VOLUME_DOWN: u32 = 25;
    pub const POWER: u32 = 26;
    pub const CAMERA: u32 = 27;
    pub const CLEAR: u32 = 28;
    pub const ENTER: u32 = 66;
    pub const DEL: u32 = 67;
    pub const TAB: u32 = 61;
    pub const SPACE: u32 = 62;
    pub const ESCAPE: u32 = 111;
    pub const MENU: u32 = 82;
    pub const APP_SWITCH: u32 = 187;
    pub const SEARCH: u32 = 84;
}

/// Get screen dimensions via `adb shell wm size`.
pub async fn get_screen_size(serial: Option<&str>) -> Result<(u32, u32)> {
    let mut cmd = Command::new("adb");

    if let Some(s) = serial {
        cmd.args(["-s", s]);
    }

    cmd.args(["shell", "wm", "size"]);

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
    // Output format: "Physical size: 1080x1920"
    for line in stdout.lines() {
        if line.contains("Physical size:") || line.contains("Override size:") {
            if let Some(size_str) = line.split(':').nth(1) {
                let size_str = size_str.trim();
                let parts: Vec<&str> = size_str.split('x').collect();
                if parts.len() == 2 {
                    let width: u32 = parts[0].parse().map_err(|_| {
                        AdbError::InvalidOutput(format!("Invalid width: {}", parts[0]))
                    })?;
                    let height: u32 = parts[1].parse().map_err(|_| {
                        AdbError::InvalidOutput(format!("Invalid height: {}", parts[1]))
                    })?;
                    return Ok((width, height));
                }
            }
        }
    }

    Err(AdbError::InvalidOutput(
        "Could not parse screen size from wm size output".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycodes() {
        assert_eq!(keycodes::HOME, 3);
        assert_eq!(keycodes::BACK, 4);
        assert_eq!(keycodes::ENTER, 66);
    }
}
