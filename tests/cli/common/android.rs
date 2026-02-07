//! Android-specific test utilities.
//!
//! Provides helper functions for Android device handling in integration tests.
//! This is the Android equivalent of the iOS common module.

#![allow(dead_code)]

use std::thread;
use std::time::Duration;

use agent_mobile_platform_android::{AdbConnection, AdbError};

/// Get a valid Android device serial number via native ADB protocol.
///
/// Returns the serial of the first connected device/emulator.
/// Panics if no Android device is available.
///
/// # Panics
/// - If ADB server is not reachable
/// - If no Android device/emulator is connected
///
/// # Example
/// ```
/// let serial = get_available_serial();
/// println!("Using Android device: {}", serial);
/// ```
pub fn get_available_serial() -> String {
    let devices = agent_mobile_platform_android::list_devices()
        .expect("Failed to list devices - ensure ADB server is running");

    for (serial, state) in &devices {
        if state == "device" {
            return serial.clone();
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
/// - If ADB connection fails
/// - If device is not responsive
pub fn ensure_device_ready(serial: &str) {
    let api_level = agent_mobile_platform_android::get_api_level(serial)
        .expect("Failed to connect to Android device");

    assert!(
        api_level.is_some(),
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
        let result = (|| -> std::result::Result<String, AdbError> {
            let mut conn = AdbConnection::new(serial)?;
            conn.shell_command("getprop sys.boot_completed")
        })();

        if let Ok(output) = result {
            if output.trim() == "1" {
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
    let result = (|| -> std::result::Result<String, AdbError> {
        let mut conn = AdbConnection::new(serial)?;
        conn.shell_command_args(&["pm", "list", "packages", package])
    })();

    match result {
        Ok(stdout) => stdout
            .lines()
            .any(|line| line.trim() == format!("package:{}", package)),
        Err(_) => false,
    }
}

/// Get Android API level for the device.
///
/// # Returns
/// API level as integer (e.g., 30 for Android 11)
pub fn get_api_level(serial: &str) -> u32 {
    agent_mobile_platform_android::get_api_level(serial)
        .expect("Failed to get API level")
        .unwrap_or_else(|| panic!("API level not available for device {}", serial))
}

/// Check if the device is an emulator.
///
/// # Returns
/// true if the device is an emulator, false if physical device
pub fn is_emulator(serial: &str) -> bool {
    agent_mobile_platform_android::is_emulator(serial)
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
