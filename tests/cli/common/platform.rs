//! Platform abstraction layer for iOS and Android device handling.
//!
//! Provides a unified interface for working with both iOS and Android devices
//! in integration tests without needing to know the platform upfront.

#![allow(dead_code)]

use super::android;
use crate::idb_common;

/// Supported test platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPlatform {
    Ios,
    Android,
}

/// Device identifier with platform detection
#[derive(Debug, Clone)]
pub struct DeviceIdentifier {
    /// Device ID (UDID for iOS, Serial for Android)
    pub id: String,
    /// Detected platform
    pub platform: TestPlatform,
}

impl DeviceIdentifier {
    /// Get any available device (tries iOS first, then Android)
    ///
    /// This is the recommended way to write platform-agnostic tests.
    ///
    /// # Panics
    /// Panics if no iOS or Android device is available.
    ///
    /// # Example
    /// ```
    /// let device = DeviceIdentifier::get_any_available()
    ///     .expect("No device available");
    /// device.ensure_ready();
    /// let bundle_id = device.get_test_bundle_id();
    /// ```
    pub fn get_any_available() -> Result<Self, String> {
        // Try iOS first
        match std::panic::catch_unwind(|| idb_common::get_available_udid()) {
            Ok(udid) => {
                return Ok(Self {
                    id: udid,
                    platform: TestPlatform::Ios,
                });
            }
            Err(_) => {
                // iOS not available, try Android
            }
        }

        // Try Android next
        match std::panic::catch_unwind(|| android::get_available_serial()) {
            Ok(serial) => Ok(Self {
                id: serial,
                platform: TestPlatform::Android,
            }),
            Err(_) => Err("No iOS or Android device available".to_string()),
        }
    }

    /// Create a DeviceIdentifier from an iOS UDID
    pub fn from_ios_udid(udid: String) -> Self {
        Self {
            id: udid,
            platform: TestPlatform::Ios,
        }
    }

    /// Create a DeviceIdentifier from an Android serial
    pub fn from_android_serial(serial: String) -> Self {
        Self {
            id: serial,
            platform: TestPlatform::Android,
        }
    }

    /// Get the test app bundle ID/package name for this platform
    pub fn get_test_bundle_id(&self) -> String {
        match self.platform {
            TestPlatform::Ios => idb_common::get_test_bundle_id(),
            TestPlatform::Android => android::get_test_package_name(),
        }
    }

    /// Ensure the device is ready for testing
    ///
    /// - iOS: Ensures companion is running
    /// - Android: Ensures device is connected and responsive
    pub fn ensure_ready(&self) {
        match self.platform {
            TestPlatform::Ios => idb_common::ensure_companion_running(&self.id),
            TestPlatform::Android => android::ensure_device_ready(&self.id),
        }
    }

    /// Get platform name as string
    pub fn platform_name(&self) -> &str {
        match self.platform {
            TestPlatform::Ios => "iOS",
            TestPlatform::Android => "Android",
        }
    }
}

/// Detect platform from device ID string
///
/// # Format Detection
/// - iOS UDIDs: 36-40 chars, alphanumeric with hyphens (e.g., "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX")
/// - Android serials: Various formats (e.g., "emulator-5554", "192.168.1.5:5555")
///
/// # Returns
/// Ok(TestPlatform) if platform can be detected, Err(String) otherwise
pub fn detect_platform(device_id: &str) -> Result<TestPlatform, String> {
    // iOS UDID pattern: typically 36-40 chars, alphanumeric with hyphens
    // Example: "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
    if device_id.len() >= 36
        && device_id.len() <= 40
        && device_id.chars().all(|c| c.is_alphanumeric() || c == '-')
        && device_id.matches('-').count() >= 3
    {
        return Ok(TestPlatform::Ios);
    }

    // Android serial patterns:
    // - emulator-XXXX
    // - IP:PORT (e.g., 192.168.1.5:5555)
    // - Physical device serials (alphanumeric, no hyphens)
    if device_id.starts_with("emulator-")
        || device_id.contains(':')
        || (device_id.len() < 36 && device_id.chars().all(|c| c.is_alphanumeric()))
    {
        return Ok(TestPlatform::Android);
    }

    Err(format!(
        "Unable to detect platform for device ID: {}",
        device_id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform_ios() {
        let udid = "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX";
        assert_eq!(detect_platform(udid).unwrap(), TestPlatform::Ios);
    }

    #[test]
    fn test_detect_platform_android_emulator() {
        let serial = "emulator-5554";
        assert_eq!(detect_platform(serial).unwrap(), TestPlatform::Android);
    }

    #[test]
    fn test_detect_platform_android_ip() {
        let serial = "192.168.1.5:5555";
        assert_eq!(detect_platform(serial).unwrap(), TestPlatform::Android);
    }

    #[test]
    fn test_detect_platform_android_physical() {
        let serial = "ABC123DEF456";
        assert_eq!(detect_platform(serial).unwrap(), TestPlatform::Android);
    }

    #[test]
    fn test_detect_platform_unknown() {
        let unknown = "???";
        assert!(detect_platform(unknown).is_err());
    }
}
