//! Internal helper for running `xcrun simctl` commands.

use std::process::Command;

use super::management::{Result, SimctlError};

/// Execute `xcrun simctl <args>` and return stdout as a String.
pub(super) fn run_simctl(args: &[&str]) -> Result<String> {
    let output = Command::new("xcrun").arg("simctl").args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output
            .status
            .code()
            .map_or("unknown".to_string(), |c| c.to_string());
        return Err(SimctlError::CommandFailed(format!(
            "simctl command failed (exit code: {}): xcrun simctl {}\nstderr: {}",
            exit_code,
            args.join(" "),
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout).map_err(|e| SimctlError::InvalidOutput(e.to_string()))
}

/// Execute `xcrun simctl <args>`, treating specific stderr patterns as success.
///
/// If the command fails but stderr contains any of `ok_patterns`, the error is
/// suppressed and an empty string is returned.
pub(super) fn run_simctl_tolerant(args: &[&str], ok_patterns: &[&str]) -> Result<String> {
    let output = Command::new("xcrun").arg("simctl").args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if ok_patterns.iter().any(|p| stderr.contains(p)) {
            return Ok(String::new());
        }
        let exit_code = output
            .status
            .code()
            .map_or("unknown".to_string(), |c| c.to_string());
        return Err(SimctlError::CommandFailed(format!(
            "simctl command failed (exit code: {}): xcrun simctl {}\nstderr: {}",
            exit_code,
            args.join(" "),
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout).map_err(|e| SimctlError::InvalidOutput(e.to_string()))
}
