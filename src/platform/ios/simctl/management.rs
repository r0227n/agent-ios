//! Simulator management commands via xcrun simctl.
//!
//! This module provides direct access to simulator lifecycle operations
//! without going through idb_companion.

#![allow(dead_code)]

use std::process::Command;
use thiserror::Error;

use crate::cli::core::screenshot::ImageFormat;

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

    let data = std::fs::read(&temp_path)
        .map_err(|e| SimctlError::InvalidOutput(format!("Failed to read screenshot: {}", e)))?;

    let _ = std::fs::remove_file(&temp_path);

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simctl_error_display() {
        let err = SimctlError::CommandFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }
}
