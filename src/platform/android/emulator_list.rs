//! Android emulator listing and device suggestion.
//!
//! This module provides functionality for listing available Android emulators
//! and suggesting the most suitable ones based on scoring criteria.
//!
//! # Architecture
//!
//! ```text
//! adb devices + emulator -list-avds
//!          ↓
//!    EmulatorLister::list_devices()
//!          ↓
//!    Vec<EmulatorDevice>
//!          ↓
//!    suggest_devices()
//!          ↓
//!    Vec<EmulatorDevice> (sorted by score)
//!          ↓
//!    DeviceSuggester trait implementation
//! ```
//!
//! # Key Types
//!
//! | Type | Role |
//! |------|------|
//! | [`EmulatorDevice`] | Individual device info (name, serial, state, API level) |
//! | [`EmulatorLister`] | Main entry point for listing and suggesting devices |
//!
//! # Scoring Logic
//!
//! | Condition | Score |
//! |-----------|-------|
//! | Online (running) | +15 |
//! | Available | +5 |
//! | API 34+ (Android 14+) | +5 |
//! | API 33 (Android 13) | +3 |
//! | API 30-32 | +1 |
//! | Emulator | +2 |
//! | Pixel device | +3 |
//! | Latest Pixel (6/7/8/9) | +2 |
//! | Phone form factor | +1 |

use super::super::{DeviceSuggester, DeviceSuggestion};
use super::adb;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Device type enumeration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceType {
    Emulator,
    Physical,
}

/// Android emulator/device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorDevice {
    pub name: String,
    pub serial: String,
    pub state: String,
    pub api_level: Option<u32>,
    pub android_version: Option<String>,
    pub device_type: DeviceType,
    pub avd_name: Option<String>,
    pub is_available: bool,
}

/// Emulator lister for Android devices.
pub struct EmulatorLister {}

impl EmulatorLister {
    /// Create a new emulator lister.
    pub fn new() -> Result<Self, adb::AdbError> {
        if !adb::is_adb_available() {
            return Err(adb::AdbError::AdbNotFound);
        }
        Ok(Self {})
    }

    /// List all connected devices and available AVDs.
    pub fn list_devices(
        &self,
    ) -> Result<Vec<EmulatorDevice>, Box<dyn std::error::Error + Send + Sync>> {
        let mut devices = Vec::new();

        // Get connected devices
        let connected = adb::list_devices()?;
        for (serial, state) in connected {
            let device = self.build_device_info(&serial, &state)?;
            devices.push(device);
        }

        // Get available AVDs (not currently running)
        let running_avds: Vec<String> = devices.iter().filter_map(|d| d.avd_name.clone()).collect();

        if let Ok(avds) = adb::list_avds() {
            for avd in avds {
                // Skip if already running
                if running_avds.iter().any(|r| r == &avd) {
                    continue;
                }

                devices.push(EmulatorDevice {
                    name: avd.clone(),
                    serial: String::new(),
                    state: "Shutdown".to_string(),
                    api_level: None,
                    android_version: None,
                    device_type: DeviceType::Emulator,
                    avd_name: Some(avd),
                    is_available: true,
                });
            }
        }

        Ok(devices)
    }

    /// Build device info by querying device properties.
    fn build_device_info(
        &self,
        serial: &str,
        state: &str,
    ) -> Result<EmulatorDevice, Box<dyn std::error::Error + Send + Sync>> {
        let is_online = state == "device";
        let is_emulator = adb::is_emulator(serial);

        // Get device properties if online
        let (api_level, android_version, model, avd_name) = if is_online {
            (
                adb::get_api_level(serial).ok().flatten(),
                adb::get_android_version(serial).ok().flatten(),
                adb::get_device_model(serial).ok().flatten(),
                if is_emulator {
                    adb::get_avd_name(serial).ok().flatten()
                } else {
                    None
                },
            )
        } else {
            (None, None, None, None)
        };

        // Determine display name
        let name = avd_name
            .clone()
            .or(model)
            .unwrap_or_else(|| serial.to_string());

        let state_str = match state {
            "device" => "Online",
            "offline" => "Offline",
            "unauthorized" => "Unauthorized",
            _ => state,
        };

        Ok(EmulatorDevice {
            name,
            serial: serial.to_string(),
            state: state_str.to_string(),
            api_level,
            android_version,
            device_type: if is_emulator {
                DeviceType::Emulator
            } else {
                DeviceType::Physical
            },
            avd_name,
            is_available: is_online,
        })
    }

    /// Get device recommendations with scoring.
    pub fn suggest_devices(
        &self,
        limit: usize,
    ) -> Result<Vec<EmulatorDevice>, Box<dyn std::error::Error + Send + Sync>> {
        let devices = self.list_devices()?;

        // Score and sort devices
        let mut scored: Vec<(i32, EmulatorDevice)> =
            devices.into_iter().map(|d| (score_device(&d), d)).collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        Ok(scored.into_iter().take(limit).map(|(_, d)| d).collect())
    }
}

impl Default for EmulatorLister {
    fn default() -> Self {
        Self::new().expect("Failed to create EmulatorLister")
    }
}

/// Score a device for recommendations.
fn score_device(device: &EmulatorDevice) -> i32 {
    let mut score = 0;

    // Online preferred (+15)
    if device.state == "Online" {
        score += 15;
    }

    // Available preferred (+5)
    if device.is_available {
        score += 5;
    }

    // API level scoring
    if let Some(api) = device.api_level {
        if api >= 34 {
            // Android 14+
            score += 5;
        } else if api == 33 {
            // Android 13
            score += 3;
        } else if api >= 30 {
            // Android 11-12
            score += 1;
        }
    }

    // Emulator preferred for testing (+2)
    if device.device_type == DeviceType::Emulator {
        score += 2;
    }

    // Pixel device preferred (+3)
    let name_lower = device.name.to_lowercase();
    if name_lower.contains("pixel") {
        score += 3;

        // Latest Pixel models (+2)
        if name_lower.contains("pixel 6")
            || name_lower.contains("pixel 7")
            || name_lower.contains("pixel 8")
            || name_lower.contains("pixel 9")
        {
            score += 2;
        }
    }

    // Phone form factor (+1) - not tablet/fold/watch
    if !name_lower.contains("tablet")
        && !name_lower.contains("fold")
        && !name_lower.contains("watch")
        && !name_lower.contains("tv")
    {
        score += 1;
    }

    score
}

/// Generate scoring reasons for a device.
fn generate_reasons(device: &EmulatorDevice) -> Vec<String> {
    let mut reasons = Vec::new();

    if device.state == "Online" {
        reasons.push("Currently running".to_string());
    }

    if let Some(api) = device.api_level {
        if api >= 34 {
            reasons.push("Latest Android".to_string());
        } else if api >= 33 {
            reasons.push("Recent Android".to_string());
        }
    }

    let name_lower = device.name.to_lowercase();
    if name_lower.contains("pixel") {
        reasons.push("Pixel device".to_string());
    }

    if device.device_type == DeviceType::Emulator {
        reasons.push("Emulator".to_string());
    }

    reasons
}

#[async_trait]
impl DeviceSuggester for EmulatorLister {
    async fn suggest(
        &self,
        count: usize,
    ) -> Result<Vec<DeviceSuggestion>, Box<dyn std::error::Error + Send + Sync>> {
        let devices = self.suggest_devices(count)?;

        // Convert EmulatorDevice to DeviceSuggestion
        let mut suggestions: Vec<DeviceSuggestion> = devices
            .into_iter()
            .map(|d| {
                let score = score_device(&d) as f64;
                let reasons = generate_reasons(&d);

                let os_version = match (&d.android_version, d.api_level) {
                    (Some(ver), Some(api)) => Some(format!("Android {} (API {})", ver, api)),
                    (Some(ver), None) => Some(format!("Android {}", ver)),
                    (None, Some(api)) => Some(format!("API {}", api)),
                    (None, None) => None,
                };

                // Use serial for online devices, AVD name for shutdown AVDs
                let identifier = if d.serial.is_empty() {
                    d.avd_name.clone().unwrap_or_default()
                } else {
                    d.serial.clone()
                };

                DeviceSuggestion {
                    name: d.name,
                    udid: identifier,
                    os_version,
                    state: Some(d.state),
                    platform: "android".to_string(),
                    score,
                    reasons,
                }
            })
            .collect();

        // Add "Recommended" to first item
        if let Some(first) = suggestions.first_mut() {
            first.reasons.insert(0, "Recommended".to_string());
        }

        Ok(suggestions)
    }

    fn platform_name(&self) -> &'static str {
        "android"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_device_online_pixel() {
        let device = EmulatorDevice {
            name: "Pixel 8 Pro".to_string(),
            serial: "emulator-5554".to_string(),
            state: "Online".to_string(),
            api_level: Some(34),
            android_version: Some("14".to_string()),
            device_type: DeviceType::Emulator,
            avd_name: Some("Pixel_8_Pro".to_string()),
            is_available: true,
        };

        let score = score_device(&device);
        // Online (15) + Available (5) + API 34+ (5) + Emulator (2) + Pixel (3) + Latest Pixel (2) + Phone (1) = 33
        assert_eq!(score, 33);
    }

    #[test]
    fn test_score_device_shutdown() {
        let device = EmulatorDevice {
            name: "Pixel 7".to_string(),
            serial: String::new(),
            state: "Shutdown".to_string(),
            api_level: None,
            android_version: None,
            device_type: DeviceType::Emulator,
            avd_name: Some("Pixel_7".to_string()),
            is_available: true,
        };

        let score = score_device(&device);
        // Available (5) + Emulator (2) + Pixel (3) + Latest Pixel (2) + Phone (1) = 13
        assert_eq!(score, 13);
    }

    #[test]
    fn test_score_device_physical() {
        let device = EmulatorDevice {
            name: "Samsung Galaxy S23".to_string(),
            serial: "RF8N1234567".to_string(),
            state: "Online".to_string(),
            api_level: Some(33),
            android_version: Some("13".to_string()),
            device_type: DeviceType::Physical,
            avd_name: None,
            is_available: true,
        };

        let score = score_device(&device);
        // Online (15) + Available (5) + API 33 (3) + Phone (1) = 24
        assert_eq!(score, 24);
    }

    #[test]
    fn test_generate_reasons() {
        let device = EmulatorDevice {
            name: "Pixel 8".to_string(),
            serial: "emulator-5554".to_string(),
            state: "Online".to_string(),
            api_level: Some(34),
            android_version: Some("14".to_string()),
            device_type: DeviceType::Emulator,
            avd_name: None,
            is_available: true,
        };

        let reasons = generate_reasons(&device);
        assert!(reasons.contains(&"Currently running".to_string()));
        assert!(reasons.contains(&"Latest Android".to_string()));
        assert!(reasons.contains(&"Pixel device".to_string()));
        assert!(reasons.contains(&"Emulator".to_string()));
    }
}
