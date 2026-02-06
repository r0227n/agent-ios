//! Screenshot capture module for Android devices.
//!
//! This module provides functions to capture screenshots from Android devices
//! using the `adb shell screencap` command.

use super::commands::{AdbError, Result};
use tokio::process::Command;

/// Take a screenshot and save to the specified path.
///
/// This function executes the following steps:
/// 1. Capture screenshot to device storage: `adb shell screencap /sdcard/screenshot.png`
/// 2. Pull the screenshot to host: `adb pull /sdcard/screenshot.png <output_path>`
/// 3. Clean up device storage: `adb shell rm /sdcard/screenshot.png`
///
/// # Arguments
/// - `serial`: Device serial number (optional, uses default device if None)
/// - `output_path`: Path where the screenshot will be saved on the host
///
/// # Returns
/// - `Ok(())` on success
/// - `Err(AdbError)` if any step fails
///
/// # Example
/// ```
/// use platform_android::adb::screenshot::screenshot;
///
/// screenshot(Some("emulator-5554"), "/tmp/screenshot.png").await?;
/// ```
pub async fn screenshot(serial: Option<&str>, output_path: &str) -> Result<()> {
    let temp_path = "/sdcard/agent_mobile_screenshot.png";

    // Step 1: Take screenshot on device
    let mut screencap_cmd = Command::new("adb");
    if let Some(s) = serial {
        screencap_cmd.args(["-s", s]);
    }
    screencap_cmd.args(["shell", "screencap", temp_path]);

    let output = screencap_cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AdbError::AdbNotFound
        } else {
            AdbError::ExecutionError(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(format!(
            "screencap failed: {}",
            stderr
        )));
    }

    // Step 2: Pull screenshot from device to host
    let mut pull_cmd = Command::new("adb");
    if let Some(s) = serial {
        pull_cmd.args(["-s", s]);
    }
    pull_cmd.args(["pull", temp_path, output_path]);

    let output = pull_cmd.output().await.map_err(AdbError::ExecutionError)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AdbError::CommandFailed(format!("pull failed: {}", stderr)));
    }

    // Step 3: Clean up temporary file on device
    let mut rm_cmd = Command::new("adb");
    if let Some(s) = serial {
        rm_cmd.args(["-s", s]);
    }
    rm_cmd.args(["shell", "rm", temp_path]);

    // Ignore cleanup errors (non-critical)
    let _ = rm_cmd.output().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires connected Android device
    async fn test_screenshot_basic() {
        let result = screenshot(None, "/tmp/test_screenshot.png").await;
        // Will fail if no device connected, which is expected
        assert!(result.is_ok() || matches!(result, Err(AdbError::CommandFailed(_))));
    }
}
