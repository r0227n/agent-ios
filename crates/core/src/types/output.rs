//! Output formatting utilities.

use super::target::{Address, DeviceInfo};
use serde_json::{json, Value};

/// Human-readable format for device list output:
/// "{name} | {udid} | {state} | {target_type} | {os_version} | {architecture} | {companion_address}"
pub fn human_format_target(target: &DeviceInfo) -> String {
    let companion_str = match &target.companion_info {
        Some(info) => info.address.to_string(),
        None => "No Companion Connected".to_string(),
    };

    format!(
        "{} | {} | {} | {} | {} | {} | {}",
        target.name,
        target.udid,
        target.state.as_deref().unwrap_or("Unknown"),
        target.target_type.as_str(),
        target.os_version.as_deref().unwrap_or("Unknown"),
        target.architecture.as_deref().unwrap_or("Unknown"),
        companion_str,
    )
}

/// JSON format for device list output
pub fn json_format_target(target: &DeviceInfo) -> String {
    let mut data: Value = json!({
        "name": target.name,
        "udid": target.udid,
        "state": target.state,
        "type": target.target_type.as_str(),
        "os_version": target.os_version,
        "architecture": target.architecture,
    });

    if let Some(companion) = &target.companion_info {
        match &companion.address {
            Address::Tcp { host, port } => {
                data["host"] = json!(host);
                data["port"] = json!(port);
                data["is_local"] = json!(companion.is_local);
                data["companion"] = json!(format!("{}:{}", host, port));
            }
            Address::DomainSocket { path } => {
                data["path"] = json!(path);
                data["is_local"] = json!(true);
                data["companion"] = json!(path);
            }
        }
    }

    serde_json::to_string(&data).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::target::{CompanionInfo, TargetType};

    #[test]
    fn test_human_format_with_companion() {
        let target = DeviceInfo {
            name: "iPhone 14 Pro".to_string(),
            udid: "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE".to_string(),
            state: Some("Booted".to_string()),
            target_type: TargetType::Simulator,
            os_version: Some("iOS 17.0".to_string()),
            architecture: Some("arm64".to_string()),
            companion_info: Some(CompanionInfo {
                udid: "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE".to_string(),
                is_local: true,
                pid: None,
                address: Address::DomainSocket {
                    path: "/tmp/test_companion.sock".to_string(),
                },
            }),
        };

        let output = human_format_target(&target);
        assert_eq!(
            output,
            "iPhone 14 Pro | AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE | Booted | simulator | iOS 17.0 | arm64 | /tmp/test_companion.sock"
        );
    }

    #[test]
    fn test_human_format_without_companion() {
        let target = DeviceInfo {
            name: "iPhone 14 Pro".to_string(),
            udid: "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE".to_string(),
            state: Some("Booted".to_string()),
            target_type: TargetType::Simulator,
            os_version: Some("iOS 17.0".to_string()),
            architecture: Some("arm64".to_string()),
            companion_info: None,
        };

        let output = human_format_target(&target);
        assert!(output.contains("No Companion Connected"));
    }

    #[test]
    fn test_json_format_with_domain_socket() {
        let target = DeviceInfo {
            name: "iPhone 14 Pro".to_string(),
            udid: "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE".to_string(),
            state: Some("Booted".to_string()),
            target_type: TargetType::Simulator,
            os_version: Some("iOS 17.0".to_string()),
            architecture: Some("arm64".to_string()),
            companion_info: Some(CompanionInfo {
                udid: "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE".to_string(),
                is_local: true,
                pid: None,
                address: Address::DomainSocket {
                    path: "/tmp/test_companion.sock".to_string(),
                },
            }),
        };

        let output = json_format_target(&target);
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["name"], "iPhone 14 Pro");
        assert_eq!(parsed["type"], "simulator");
        assert_eq!(parsed["path"], "/tmp/test_companion.sock");
        assert_eq!(parsed["is_local"], true);
        assert_eq!(parsed["companion"], "/tmp/test_companion.sock");
    }
}
