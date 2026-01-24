//! Platform detection and device resolution

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
        if Self::has_android_devices().await {
            return Ok(Platform::Android);
        }
        Err(
            "No device found. Please connect an iOS simulator/device or Android emulator/device."
                .into(),
        )
    }

    /// Check if iOS devices are available
    pub async fn has_ios_devices() -> bool {
        use std::path::Path;
        let state_path = Path::new("/tmp/idb/state");
        if !state_path.exists() {
            return false;
        }
        if let Ok(content) = std::fs::read_to_string(state_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = json.as_array() {
                    return !arr.is_empty();
                }
            }
        }
        false
    }

    /// Check if Android devices are available
    pub async fn has_android_devices() -> bool {
        use tokio::process::Command;
        let output = Command::new("adb").args(["devices", "-l"]).output().await;
        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.lines().skip(1).any(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.len() >= 2 && parts[1] == "device"
                })
            }
            _ => false,
        }
    }
}
