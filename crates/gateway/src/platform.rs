//! Platform detection and device resolution
//!
//! Uses native ADB protocol for Android device detection.

use agent_mobile_core::Platform;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Device resolver for platform detection
pub struct DeviceResolver;

impl DeviceResolver {
    /// Parse platform from string
    pub fn parse_platform(s: &str) -> Result<Platform> {
        s.parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })
    }

    /// Detect platform based on available devices
    pub async fn detect_platform() -> Result<Platform> {
        if Self::has_ios_devices().await {
            return Ok(Platform::Ios);
        }
        if Self::has_android_devices() {
            return Ok(Platform::Android);
        }
        Err(
            "No device found. Please connect an iOS simulator/device or Android emulator/device."
                .into(),
        )
    }

    /// Check if iOS devices are available
    pub async fn has_ios_devices() -> bool {
        use tokio::process::Command;
        let output = Command::new("xcrun")
            .args(["simctl", "list", "devices", "-j"])
            .output()
            .await;
        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(devices) = json.get("devices").and_then(|d| d.as_object()) {
                        return devices.values().any(|list| {
                            list.as_array().is_some_and(|arr| {
                                arr.iter().any(|d| {
                                    d.get("state").and_then(|s| s.as_str()) == Some("Booted")
                                })
                            })
                        });
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check if Android devices are available via native ADB protocol.
    pub fn has_android_devices() -> bool {
        if !agent_mobile_platform_android::is_adb_available() {
            return false;
        }
        match agent_mobile_platform_android::list_devices() {
            Ok(devices) => devices.iter().any(|(_, state)| state == "device"),
            Err(_) => false,
        }
    }
}
