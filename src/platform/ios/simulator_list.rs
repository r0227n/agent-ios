//! iOS simulator listing and device suggestion.
//!
//! This module provides functionality for listing available iOS simulators
//! and suggesting the most suitable ones based on scoring criteria.
//!
//! # Architecture
//!
//! ```text
//! xcrun simctl list devices --json
//!          ↓
//!    SimulatorLister::list_simulators()
//!          ↓
//!    Vec<SimulatorDevice>
//!          ↓
//!    suggest_simulators()
//!          ↓
//!    Vec<SimulatorDevice> (sorted by score)
//!          ↓
//!    DeviceSuggester trait implementation
//! ```
//!
//! # Key Types
//!
//! | Type | Role |
//! |------|------|
//! | [`SimulatorDevice`] | Individual simulator info (name, UDID, state, runtime) |
//! | [`SimulatorLister`] | Main entry point for listing and suggesting simulators |
//!
//! # Scoring Logic
//!
//! `suggest_simulators()` scores devices based on the following criteria:
//!
//! | Condition | Score |
//! |-----------|-------|
//! | Booted (running) | +10 |
//! | Available | +5 |
//! | iOS 18 | +3 |
//! | iOS 17 | +2 |
//! | iPhone | +1 |
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::platform::ios::simulator_list::SimulatorLister;
//!
//! // Get recommended simulators
//! let lister = SimulatorLister::new()?;
//! let recommended = lister.suggest_simulators(3)?;
//! ```

use super::super::{DeviceSuggester, DeviceSuggestion};
use super::simctl;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simulator device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorDevice {
    pub name: String,
    pub udid: String,
    pub state: String,
    pub runtime: String,
    pub is_available: bool,
}

/// Simulator lister with progressive disclosure.
pub struct SimulatorLister {}

impl SimulatorLister {
    /// Create a new simulator lister.
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {})
    }

    /// List all simulators using `xcrun simctl list devices --json`.
    pub fn list_simulators(
        &self,
    ) -> Result<Vec<SimulatorDevice>, Box<dyn std::error::Error + Send + Sync>> {
        let json_str = simctl::list_devices_json()?;
        let data: SimctlOutput = serde_json::from_str(&json_str)?;

        Ok(self.parse_devices(data))
    }

    /// Parse simctl output into flat device list.
    fn parse_devices(&self, data: SimctlOutput) -> Vec<SimulatorDevice> {
        let mut devices = Vec::new();

        for (runtime_str, device_list) in data.devices {
            // Extract runtime name (e.g., "com.apple.CoreSimulator.SimRuntime.iOS-18-1" -> "iOS 18.1")
            let runtime_name = parse_runtime_name(&runtime_str);

            for device in device_list {
                devices.push(SimulatorDevice {
                    name: device.name,
                    udid: device.udid,
                    state: device.state,
                    runtime: runtime_name.clone(),
                    is_available: device.is_available.unwrap_or(false),
                });
            }
        }

        devices
    }

    /// Get simulator recommendations with scoring.
    pub fn suggest_simulators(
        &self,
        limit: usize,
    ) -> Result<Vec<SimulatorDevice>, Box<dyn std::error::Error + Send + Sync>> {
        let devices = self.list_simulators()?;

        // Score and sort devices
        let mut scored: Vec<(i32, SimulatorDevice)> =
            devices.into_iter().map(|d| (score_device(&d), d)).collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));

        Ok(scored.into_iter().take(limit).map(|(_, d)| d).collect())
    }
}

impl Default for SimulatorLister {
    fn default() -> Self {
        Self::new().expect("Failed to create SimulatorLister")
    }
}

/// Score a device for recommendations (same logic as Python version).
fn score_device(device: &SimulatorDevice) -> i32 {
    let mut score = 0;

    // Booted preferred (+10)
    if device.state == "Booted" {
        score += 10;
    }

    // Available preferred (+5)
    if device.is_available {
        score += 5;
    }

    // iOS 18 preferred (+3), iOS 17 (+2)
    if device.runtime.contains("18") {
        score += 3;
    } else if device.runtime.contains("17") {
        score += 2;
    }

    // iPhone preferred (+1)
    if device.name.contains("iPhone") {
        score += 1;
    }

    score
}

/// Parse runtime identifier to human-readable name.
fn parse_runtime_name(runtime_str: &str) -> String {
    // Format: "com.apple.CoreSimulator.SimRuntime.iOS-18-1" -> "iOS 18.1"
    // Or: "iOS 18.1 Simulator" -> "iOS 18.1"
    if let Some(suffix) = runtime_str.strip_prefix("com.apple.CoreSimulator.SimRuntime.") {
        // iOS-18-1 -> iOS 18.1
        let parts: Vec<&str> = suffix.split('-').collect();
        if parts.len() >= 2 {
            let platform = parts[0];
            let version = parts[1..].join(".");
            return format!("{} {}", platform, version);
        }
        return suffix.to_string();
    }

    // Remove " Simulator" suffix if present
    runtime_str
        .strip_suffix(" Simulator")
        .unwrap_or(runtime_str)
        .to_string()
}

/// Simctl JSON output structure.
#[derive(Debug, Deserialize)]
struct SimctlOutput {
    devices: HashMap<String, Vec<SimctlDevice>>,
}

/// Single device from simctl output.
#[derive(Debug, Deserialize)]
struct SimctlDevice {
    name: String,
    udid: String,
    state: String,
    #[serde(rename = "isAvailable")]
    is_available: Option<bool>,
}

// DeviceSuggester trait implementation for compatibility
#[async_trait]
impl DeviceSuggester for SimulatorLister {
    async fn suggest(
        &self,
        count: usize,
    ) -> Result<Vec<DeviceSuggestion>, Box<dyn std::error::Error + Send + Sync>> {
        let devices = self.suggest_simulators(count)?;

        // Convert SimulatorDevice to DeviceSuggestion
        let mut suggestions: Vec<DeviceSuggestion> = devices
            .into_iter()
            .map(|d| {
                let score = score_device(&d) as f64;
                let mut reasons = Vec::new();

                if d.state == "Booted" {
                    reasons.push("Currently running".to_string());
                }
                if d.runtime.contains("18") {
                    reasons.push("Latest iOS".to_string());
                }
                if d.name.contains("iPhone") {
                    reasons.push("iPhone model".to_string());
                }

                DeviceSuggestion {
                    name: d.name,
                    udid: d.udid,
                    os_version: Some(d.runtime),
                    state: Some(d.state),
                    platform: "ios".to_string(),
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
        "ios"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_device() {
        let device = SimulatorDevice {
            name: "iPhone 16 Pro".to_string(),
            udid: "test-udid".to_string(),
            state: "Booted".to_string(),
            runtime: "iOS 18.1".to_string(),
            is_available: true,
        };

        let score = score_device(&device);
        // Booted (10) + Available (5) + iOS 18 (3) + iPhone (1) = 19
        assert_eq!(score, 19);
    }

    #[test]
    fn test_score_device_shutdown() {
        let device = SimulatorDevice {
            name: "iPad Pro".to_string(),
            udid: "test-udid".to_string(),
            state: "Shutdown".to_string(),
            runtime: "iOS 17.0".to_string(),
            is_available: true,
        };

        let score = score_device(&device);
        // Available (5) + iOS 17 (2) = 7
        assert_eq!(score, 7);
    }

    #[test]
    fn test_parse_runtime_name() {
        assert_eq!(
            parse_runtime_name("com.apple.CoreSimulator.SimRuntime.iOS-18-1"),
            "iOS 18.1"
        );
        assert_eq!(
            parse_runtime_name("com.apple.CoreSimulator.SimRuntime.tvOS-18-0"),
            "tvOS 18.0"
        );
        assert_eq!(parse_runtime_name("iOS 18.1 Simulator"), "iOS 18.1");
        assert_eq!(parse_runtime_name("iOS 18.1"), "iOS 18.1");
    }
}
