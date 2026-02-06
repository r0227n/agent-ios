//! Screenshot capture module for Android devices.
//!
//! This module provides functions to capture screenshots from Android devices
//! using the native ADB protocol by writing a temporary file on-device and
//! pulling it, avoiding stdout corruption from `screencap -p`.

use super::commands::{AdbError, Result};
use super::connection::AdbConnection;

/// Take a screenshot and save to the specified path.
///
/// Uses `screencap <path>` to write a PNG to device storage, then pulls the file via ADB.
/// This avoids binary corruption issues caused by `screencap -p` stdout pipe LF→CRLF conversion.
///
/// # Arguments
/// - `serial`: Device serial number (optional, uses default device if None)
/// - `output_path`: Path where the screenshot will be saved on the host
///
/// # Returns
/// - `Ok(())` on success
/// - `Err(AdbError)` if capture fails
pub async fn screenshot(serial: Option<&str>, output_path: &str) -> Result<()> {
    let png_bytes = screenshot_bytes(serial)?;
    std::fs::write(output_path, &png_bytes).map_err(AdbError::ExecutionError)?;
    Ok(())
}

/// Capture a screenshot and return the raw PNG bytes.
///
/// Uses `screencap <path>` to save a PNG to device storage, then pulls it via ADB.
/// This avoids binary corruption issues that can occur with `screencap -p` piped through shell.
pub fn screenshot_bytes(serial: Option<&str>) -> Result<Vec<u8>> {
    let mut conn = AdbConnection::for_device(serial)?;

    // Use a temporary file on device to avoid shell pipe corruption
    let remote_path = "/sdcard/agent_mobile_screenshot.png";

    // Capture screenshot to file (without -p flag to avoid stdout corruption)
    let output = conn.shell_command_args(&["screencap", remote_path])?;
    let output_lower = output.to_lowercase();
    if output_lower.contains("error") || output_lower.contains("failed") {
        return Err(AdbError::CommandFailed(format!(
            "screencap failed: {}",
            output
        )));
    }

    // Pull the file
    let mut png_bytes = Vec::new();
    conn.pull(remote_path, &mut png_bytes)?;

    // Clean up temporary file
    let _ = conn.shell_command_args(&["rm", remote_path]);

    if png_bytes.is_empty() {
        return Err(AdbError::CommandFailed(
            "screencap produced empty file".to_string(),
        ));
    }

    Ok(png_bytes)
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
