//! Simulator management commands via xcrun simctl.
//!
//! This module provides direct access to simulator lifecycle operations
//! without going through idb_companion.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simctl_error_display() {
        let err = SimctlError::CommandFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }
}
