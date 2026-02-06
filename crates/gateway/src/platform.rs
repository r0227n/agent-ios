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
        if Self::has_booted_ios_simulators().await {
            return Ok(Platform::Ios);
        }
        let has_android = tokio::task::spawn_blocking(Self::has_android_devices)
            .await
            .unwrap_or(false);
        if has_android {
            return Ok(Platform::Android);
        }
        Err(
            "No device found. Please connect an iOS simulator/device or Android emulator/device."
                .into(),
        )
    }

    /// Check if any iOS simulators are currently booted
    pub async fn has_booted_ios_simulators() -> bool {
        let result =
            tokio::task::spawn_blocking(|| agent_mobile_platform_ios::simctl::list_simulators())
                .await;

        match result {
            Ok(Ok(devices)) => devices.iter().any(|d| d.state.as_deref() == Some("Booted")),
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
