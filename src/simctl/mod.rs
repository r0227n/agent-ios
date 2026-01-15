use crate::types::{TargetDescription, TargetType};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct SimctlOutput {
    devices: HashMap<String, Vec<SimctlDevice>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SimctlDevice {
    name: String,
    udid: String,
    state: String,
    is_available: bool,
}

/// Extract OS version from runtime identifier
/// e.g. "com.apple.CoreSimulator.SimRuntime.iOS-26-1" -> "iOS 26.1"
fn extract_os_version(runtime: &str) -> String {
    // Remove the prefix
    let version_part = runtime
        .strip_prefix("com.apple.CoreSimulator.SimRuntime.")
        .unwrap_or(runtime);

    // Convert format: "iOS-26-1" -> "iOS 26.1"
    let parts: Vec<&str> = version_part.splitn(2, '-').collect();
    if parts.len() == 2 {
        let os_type = parts[0]; // "iOS" or "tvOS" or "watchOS"
        let version = parts[1].replace('-', "."); // "26-1" -> "26.1"
        format!("{} {}", os_type, version)
    } else {
        version_part.to_string()
    }
}

/// List all available simulators using simctl
pub fn list_simulators() -> Result<Vec<TargetDescription>, Box<dyn std::error::Error>> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "--json"])
        .output()?;

    if !output.status.success() {
        return Err(format!("simctl failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    let simctl_output: SimctlOutput = serde_json::from_slice(&output.stdout)?;

    let mut targets = Vec::new();

    for (runtime, devices) in simctl_output.devices {
        let os_version = extract_os_version(&runtime);

        for device in devices {
            if !device.is_available {
                continue;
            }

            // Determine architecture - simulators generally report x86_64
            // even on ARM Macs when running via Rosetta
            let architecture = "x86_64".to_string();

            targets.push(TargetDescription {
                name: device.name,
                udid: device.udid,
                state: Some(device.state),
                target_type: TargetType::Simulator,
                os_version: Some(os_version.clone()),
                architecture: Some(architecture),
                companion_info: None, // No companion connected for local simulators
            });
        }
    }

    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_os_version() {
        assert_eq!(
            extract_os_version("com.apple.CoreSimulator.SimRuntime.iOS-26-1"),
            "iOS 26.1"
        );
        assert_eq!(
            extract_os_version("com.apple.CoreSimulator.SimRuntime.tvOS-18-0"),
            "tvOS 18.0"
        );
        assert_eq!(
            extract_os_version("com.apple.CoreSimulator.SimRuntime.watchOS-11-2"),
            "watchOS 11.2"
        );
    }
}
