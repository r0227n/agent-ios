//! iOS environment checks for the doctor command.
//!
//! Checks Xcode CLI Tools, simctl, and idb_companion.

use std::process::Command;

use super::types::{CheckResult, PlatformResults};

/// Run all iOS environment checks.
pub fn check_all() -> PlatformResults {
    let mut results = PlatformResults::new();

    results.add(check_xcode_cli());
    results.add(check_simctl());
    results.add(check_idb_companion());

    results
}

/// Check if Xcode CLI Tools are installed.
fn check_xcode_cli() -> CheckResult {
    let output = Command::new("xcrun").arg("--version").output();

    match output {
        Ok(out) if out.status.success() => {
            let version_str = String::from_utf8_lossy(&out.stdout);
            let version = version_str.trim().to_string();
            CheckResult::ok_with_version("Xcode CLI Tools", version)
        }
        Ok(_) => CheckResult::error("Xcode CLI Tools", "xcrun returned non-zero exit code")
            .with_hint("Install: xcode-select --install"),
        Err(_) => CheckResult::error("Xcode CLI Tools", "xcrun not found")
            .with_hint("Install: xcode-select --install"),
    }
}

/// Check if simctl is available.
fn check_simctl() -> CheckResult {
    let output = Command::new("xcrun").args(["simctl", "help"]).output();

    match output {
        Ok(out) if out.status.success() => CheckResult::ok("simctl"),
        Ok(_) => CheckResult::error("simctl", "simctl returned non-zero exit code")
            .with_hint("Ensure Xcode is properly installed"),
        Err(_) => CheckResult::error("simctl", "simctl not found")
            .with_hint("Install Xcode from App Store"),
    }
}

/// Check if idb_companion is installed.
fn check_idb_companion() -> CheckResult {
    // First check if idb_companion exists
    let which_output = Command::new("which").arg("idb_companion").output();

    match which_output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();

            // Try to get version
            let version_output = Command::new("idb_companion").arg("--version").output();

            match version_output {
                Ok(ver_out) if ver_out.status.success() => {
                    let version = String::from_utf8_lossy(&ver_out.stdout).trim().to_string();
                    CheckResult::ok_with_version("idb_companion", version).with_path(path)
                }
                _ => {
                    // idb_companion exists but version check failed, still OK
                    CheckResult::ok("idb_companion").with_path(path)
                }
            }
        }
        _ => CheckResult::error("idb_companion", "not found")
            .with_hint("Install: brew install idb-companion"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_all_returns_results() {
        let results = check_all();
        assert_eq!(results.checks.len(), 3);
    }
}
