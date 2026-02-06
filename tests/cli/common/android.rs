//! Android-specific test utilities.
//!
//! Provides helper functions for Android device handling in integration tests.
//! This is the Android equivalent of the iOS common module.

#![allow(dead_code)]

use std::process::Command;
use std::thread;
use std::time::Duration;

/// Get a valid Android device serial number from adb devices.
///
/// Returns the serial of the first connected device/emulator.
/// Panics if no Android device is available.
///
/// # Panics
/// - If adb is not installed or not in PATH
/// - If no Android device/emulator is connected
///
/// # Example
/// ```
/// let serial = get_available_serial();
/// println!("Using Android device: {}", serial);
/// ```
pub fn get_available_serial() -> String {
    let output = Command::new("adb")
        .args(["devices", "-l"])
        .output()
        .expect("Failed to execute adb - ensure Android SDK is installed and adb is in PATH");

    assert!(
        output.status.success(),
        "adb devices failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        // Skip "List of devices attached" header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            return parts[0].to_string();
        }
    }

    panic!(
        "No Android device/emulator available. Run 'adb devices' to check.\n\
         Tip: Start an emulator with 'emulator -avd <name>' or connect a physical device."
    );
}

/// Ensure Android device is ready for testing.
///
/// Verifies the device is connected and responsive by checking system properties.
/// This is the Android equivalent of ensure_companion_running() for iOS.
///
/// # Panics
/// - If adb command fails
/// - If device is not responsive
pub fn ensure_device_ready(serial: &str) {
    let output = Command::new("adb")
        .args(["-s", serial, "shell", "getprop", "ro.build.version.sdk"])
        .output()
        .expect("Failed to execute adb");

    assert!(
        output.status.success(),
        "Failed to connect to Android device {}: {}",
        serial,
        String::from_utf8_lossy(&output.stderr)
    );

    let api_level = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        !api_level.is_empty(),
        "Device {} is not responding properly",
        serial
    );
}

/// Get the test app package name (Settings app is always available).
///
/// Android equivalent of get_test_bundle_id() for iOS.
///
/// # Returns
/// "com.android.settings" - The Android Settings app package name
pub fn get_test_package_name() -> String {
    "com.android.settings".to_string()
}

/// Wait for Android UI to be ready (system boot completed).
///
/// This is useful when testing immediately after emulator startup.
/// Waits up to 30 seconds for the system to finish booting.
///
/// # Panics
/// - If device doesn't become ready within timeout
pub fn wait_for_ui_ready(serial: &str) {
    const MAX_ATTEMPTS: u32 = 30;
    const RETRY_DELAY_MS: u64 = 1000;

    for attempt in 1..=MAX_ATTEMPTS {
        let output = Command::new("adb")
            .args(["-s", serial, "shell", "getprop", "sys.boot_completed"])
            .output()
            .expect("Failed to execute adb");

        if output.status.success() {
            let boot_completed = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if boot_completed == "1" {
                eprintln!("Android UI ready after {} attempt(s)", attempt);
                return;
            }
        }

        if attempt < MAX_ATTEMPTS {
            eprintln!(
                "Attempt {}/{}: Android UI not ready, retrying...",
                attempt, MAX_ATTEMPTS
            );
            thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
        }
    }

    panic!(
        "Android device {} failed to become ready after {} attempts",
        serial, MAX_ATTEMPTS
    );
}

/// Check if a package is installed on the device.
///
/// # Returns
/// true if the package is installed, false otherwise
pub fn is_package_installed(serial: &str, package: &str) -> bool {
    let output = Command::new("adb")
        .args(["-s", serial, "shell", "pm", "list", "packages", package])
        .output()
        .expect("Failed to execute adb");

    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains(&format!("package:{}", package))
}

/// Get Android API level for the device.
///
/// # Returns
/// API level as integer (e.g., 30 for Android 11)
pub fn get_api_level(serial: &str) -> u32 {
    let output = Command::new("adb")
        .args(["-s", serial, "shell", "getprop", "ro.build.version.sdk"])
        .output()
        .expect("Failed to execute adb");

    assert!(
        output.status.success(),
        "Failed to get API level: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let api_level_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    api_level_str
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse API level: {}", api_level_str))
}

/// Check if the device is an emulator.
///
/// # Returns
/// true if the device is an emulator, false if physical device
pub fn is_emulator(serial: &str) -> bool {
    serial.starts_with("emulator-") || {
        let output = Command::new("adb")
            .args(["-s", serial, "shell", "getprop", "ro.product.manufacturer"])
            .output()
            .ok();

        if let Some(output) = output {
            let manufacturer = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            manufacturer.contains("google") || manufacturer.contains("generic")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_test_package_name() {
        assert_eq!(get_test_package_name(), "com.android.settings");
    }

    #[test]
    fn test_is_emulator_serial() {
        assert!(is_emulator("emulator-5554"));
        assert!(!is_emulator("ABC123DEF456"));
    }
}
