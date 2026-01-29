//! Android environment checks for the doctor command.
//!
//! Checks ANDROID_HOME, adb, and emulator.

use std::env;
use std::process::Command;

use super::types::{CheckResult, PlatformResults};

/// Run all Android environment checks.
pub fn check_all() -> PlatformResults {
    let mut results = PlatformResults::new();

    results.add(check_android_home());
    results.add(check_adb());
    results.add(check_emulator());

    results
}

/// Check if ANDROID_HOME is set.
fn check_android_home() -> CheckResult {
    match env::var("ANDROID_HOME") {
        Ok(path) if !path.is_empty() => {
            // Verify the directory exists
            if std::path::Path::new(&path).exists() {
                CheckResult::ok_with_path("ANDROID_HOME", &path)
            } else {
                CheckResult::error(
                    "ANDROID_HOME",
                    format!("directory does not exist: {}", path),
                )
                .with_hint("Set ANDROID_HOME to a valid Android SDK path")
            }
        }
        Ok(_) => CheckResult::error("ANDROID_HOME", "environment variable is empty")
            .with_hint("Set ANDROID_HOME to your Android SDK path"),
        Err(_) => CheckResult::error("ANDROID_HOME", "environment variable not set")
            .with_hint("Set ANDROID_HOME to your Android SDK path"),
    }
}

/// Check if adb is available and get its version.
fn check_adb() -> CheckResult {
    let output = Command::new("adb").arg("version").output();

    match output {
        Ok(out) if out.status.success() => {
            let version_str = String::from_utf8_lossy(&out.stdout);
            // Parse version from output like "Android Debug Bridge version 35.0.1"
            let version = version_str
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("Android Debug Bridge version "))
                .map(|v| v.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            CheckResult::ok_with_version("adb", version)
        }
        Ok(_) => CheckResult::error("adb", "adb returned non-zero exit code")
            .with_hint("Ensure Android SDK platform-tools is in PATH"),
        Err(_) => CheckResult::error("adb", "not found")
            .with_hint("Install Android SDK and add platform-tools to PATH"),
    }
}

/// Check if emulator is available and get its version.
fn check_emulator() -> CheckResult {
    let output = Command::new("emulator").arg("-version").output();

    match output {
        Ok(out) if out.status.success() => {
            let version_str = String::from_utf8_lossy(&out.stdout);
            // Parse version from output like "emulator: Android emulator version 34.2.15"
            let version = version_str
                .lines()
                .find(|line| line.contains("Android emulator version"))
                .and_then(|line| {
                    line.split("Android emulator version ")
                        .nth(1)
                        .map(|v| v.split_whitespace().next().unwrap_or("unknown").to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());

            CheckResult::ok_with_version("emulator", version)
        }
        Ok(out) => {
            // emulator -version often returns non-zero but still works
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let combined = format!("{}{}", stdout, stderr);

            if combined.contains("Android emulator version") {
                let version = combined
                    .lines()
                    .find(|line| line.contains("Android emulator version"))
                    .and_then(|line| {
                        line.split("Android emulator version ")
                            .nth(1)
                            .map(|v| v.split_whitespace().next().unwrap_or("unknown").to_string())
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                CheckResult::ok_with_version("emulator", version)
            } else {
                CheckResult::error("emulator", "emulator returned non-zero exit code")
                    .with_hint("Ensure Android SDK emulator is in PATH")
            }
        }
        Err(_) => CheckResult::error("emulator", "not found")
            .with_hint("Install Android SDK and add emulator to PATH"),
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
