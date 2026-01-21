//! Core type definitions for agent-mobile.
//!
//! This module contains all shared types used across the codebase including
//! target descriptions, addresses, compression types, and installation artifacts.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Platform Types
// ============================================================================

/// Supported mobile platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Ios,
    Android,
}

impl Platform {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ios" => Ok(Self::Ios),
            "android" => Ok(Self::Android),
            _ => Err(format!("Invalid platform: {}. Use 'ios' or 'android'.", s)),
        }
    }
}

// ============================================================================
// Gesture Types
// ============================================================================

/// Direction for scroll/swipe gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// Get swipe coordinates for this direction (assuming center origin).
    /// Returns (start_offset, end_offset) where offset is (dx, dy) from center.
    pub fn to_swipe_offsets(&self, distance: f64) -> ((f64, f64), (f64, f64)) {
        let half = distance / 2.0;
        match self {
            // Swipe up = finger moves from bottom to top
            Self::Up => ((0.0, half), (0.0, -half)),
            // Swipe down = finger moves from top to bottom
            Self::Down => ((0.0, -half), (0.0, half)),
            // Swipe left = finger moves from right to left
            Self::Left => ((half, 0.0), (-half, 0.0)),
            // Swipe right = finger moves from left to right
            Self::Right => ((-half, 0.0), (half, 0.0)),
        }
    }
}

impl fmt::Display for ScrollDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ScrollDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(format!(
                "Invalid scroll direction: {}. Use 'up', 'down', 'left', or 'right'.",
                s
            )),
        }
    }
}

// ============================================================================
// Target Types
// ============================================================================

/// Type of iOS target (simulator, physical device, or Mac)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    Simulator,
    Device,
    Mac,
}

impl TargetType {
    /// Parse target type from proto string (handles various formats)
    pub fn from_proto_string(s: &str) -> Self {
        let normalized = s.to_lowercase();
        if normalized.contains("sim") {
            Self::Simulator
        } else if normalized.contains("dev") {
            Self::Device
        } else if normalized.contains("mac") {
            Self::Mac
        } else {
            Self::Simulator // Default fallback
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Simulator => "simulator",
            Self::Device => "device",
            Self::Mac => "mac",
        }
    }
}

impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TargetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simulator" => Ok(Self::Simulator),
            "device" => Ok(Self::Device),
            "mac" => Ok(Self::Mac),
            _ => Err(format!("Invalid target type: {}", s)),
        }
    }
}

// ============================================================================
// Address Types
// ============================================================================

/// Connection address for companion daemon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Address {
    /// TCP connection to remote companion
    Tcp { host: String, port: u16 },
    /// Unix domain socket for local companion
    DomainSocket { path: String },
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Tcp { host, port } => write!(f, "{}:{}", host, port),
            Address::DomainSocket { path } => write!(f, "{}", path),
        }
    }
}

// ============================================================================
// Companion Types
// ============================================================================

/// Information about a connected companion daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionInfo {
    pub udid: String,
    pub is_local: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub address: Address,
}

/// Description of an iOS target (device or simulator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDescription {
    pub name: String,
    pub udid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub target_type: TargetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companion_info: Option<CompanionInfo>,
}

// ============================================================================
// Installation Types
// ============================================================================

/// Result of an install operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledArtifact {
    pub name: String,
    pub uuid: Option<String>,
    pub progress: Option<f64>,
}

/// Compression type for install payloads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Gzip,
    Zstd,
}

impl std::str::FromStr for Compression {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GZIP" => Ok(Self::Gzip),
            "ZSTD" => Ok(Self::Zstd),
            _ => Err(format!(
                "Invalid compression type: {}. Use GZIP or ZSTD.",
                s
            )),
        }
    }
}

// ============================================================================
// Output Formatting
// ============================================================================

/// Human-readable format matching Python idb output:
/// "{name} | {udid} | {state} | {target_type} | {os_version} | {architecture} | {companion_address}"
pub fn human_format_target(target: &TargetDescription) -> String {
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

/// JSON format matching Python idb output
pub fn json_format_target(target: &TargetDescription) -> String {
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

// ============================================================================
// Target Merging
// ============================================================================

/// Merge local targets with connected targets.
///
/// This follows Python idb's merge_connected_targets logic:
/// - When the same UDID exists in both, prefer the connected target (with companion_info)
/// - Add any remote targets that aren't in local targets
pub fn merge_connected_targets(
    local_targets: Vec<TargetDescription>,
    connected_targets: Vec<TargetDescription>,
) -> Vec<TargetDescription> {
    let connected_map: HashMap<String, TargetDescription> = connected_targets
        .into_iter()
        .map(|t| (t.udid.clone(), t))
        .collect();

    let mut targets: HashMap<String, TargetDescription> = HashMap::new();

    // Add local targets, preferring connected version if available
    for target in local_targets {
        let udid = target.udid.clone();
        if let Some(connected) = connected_map.get(&udid) {
            targets.insert(udid, connected.clone());
        } else {
            targets.insert(udid, target);
        }
    }

    // Add any connected targets not in local
    for (udid, target) in connected_map {
        targets.entry(udid).or_insert(target);
    }

    targets.into_values().collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Target Type Tests
    #[test]
    fn test_target_type_from_str() {
        assert_eq!(
            "simulator".parse::<TargetType>().unwrap(),
            TargetType::Simulator
        );
        assert_eq!("device".parse::<TargetType>().unwrap(), TargetType::Device);
        assert_eq!("mac".parse::<TargetType>().unwrap(), TargetType::Mac);
    }

    #[test]
    fn test_target_type_from_proto_string() {
        assert_eq!(
            TargetType::from_proto_string("SIMULATOR"),
            TargetType::Simulator
        );
        assert_eq!(
            TargetType::from_proto_string("iOS Simulator"),
            TargetType::Simulator
        );
        assert_eq!(TargetType::from_proto_string("device"), TargetType::Device);
    }

    // Compression Tests
    #[test]
    fn test_compression_from_str_gzip() {
        let comp: Compression = "GZIP".parse().unwrap();
        assert_eq!(comp, Compression::Gzip);
    }

    #[test]
    fn test_compression_from_str_gzip_lowercase() {
        let comp: Compression = "gzip".parse().unwrap();
        assert_eq!(comp, Compression::Gzip);
    }

    #[test]
    fn test_compression_from_str_zstd() {
        let comp: Compression = "ZSTD".parse().unwrap();
        assert_eq!(comp, Compression::Zstd);
    }

    #[test]
    fn test_compression_from_str_invalid() {
        let result: Result<Compression, _> = "invalid".parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid compression type"));
    }

    // Output Format Tests
    fn make_test_target(with_companion: bool) -> TargetDescription {
        TargetDescription {
            name: "iPhone 14 Pro".to_string(),
            udid: "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE".to_string(),
            state: Some("Booted".to_string()),
            target_type: TargetType::Simulator,
            os_version: Some("iOS 17.0".to_string()),
            architecture: Some("arm64".to_string()),
            companion_info: if with_companion {
                Some(CompanionInfo {
                    udid: "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE".to_string(),
                    is_local: true,
                    pid: None,
                    address: Address::DomainSocket {
                        path: "/tmp/idb/some.sock".to_string(),
                    },
                })
            } else {
                None
            },
        }
    }

    #[test]
    fn test_human_format_with_companion() {
        let target = make_test_target(true);
        let output = human_format_target(&target);
        assert_eq!(
            output,
            "iPhone 14 Pro | AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE | Booted | simulator | iOS 17.0 | arm64 | /tmp/idb/some.sock"
        );
    }

    #[test]
    fn test_human_format_without_companion() {
        let target = make_test_target(false);
        let output = human_format_target(&target);
        assert!(output.contains("No Companion Connected"));
    }

    #[test]
    fn test_json_format_with_domain_socket() {
        let target = make_test_target(true);
        let output = json_format_target(&target);
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["name"], "iPhone 14 Pro");
        assert_eq!(parsed["type"], "simulator");
        assert_eq!(parsed["path"], "/tmp/idb/some.sock");
        assert_eq!(parsed["is_local"], true);
    }

    // Merge Tests
    fn make_merge_target(name: &str, udid: &str, with_companion: bool) -> TargetDescription {
        TargetDescription {
            name: name.to_string(),
            udid: udid.to_string(),
            state: Some("Booted".to_string()),
            target_type: TargetType::Simulator,
            os_version: Some("iOS 17.0".to_string()),
            architecture: Some("arm64".to_string()),
            companion_info: if with_companion {
                Some(CompanionInfo {
                    udid: udid.to_string(),
                    is_local: true,
                    pid: Some(1234),
                    address: Address::DomainSocket {
                        path: format!("/tmp/idb/{}_companion.sock", udid),
                    },
                })
            } else {
                None
            },
        }
    }

    #[test]
    fn test_merge_prefers_connected() {
        let local = vec![make_merge_target("iPhone 14", "UDID-1", false)];
        let connected = vec![make_merge_target("iPhone 14", "UDID-1", true)];

        let merged = merge_connected_targets(local, connected);

        assert_eq!(merged.len(), 1);
        assert!(merged[0].companion_info.is_some());
    }

    #[test]
    fn test_merge_adds_remote_only() {
        let local = vec![make_merge_target("iPhone 14", "UDID-1", false)];
        let connected = vec![make_merge_target("Remote Device", "UDID-2", true)];

        let merged = merge_connected_targets(local, connected);

        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_empty_inputs() {
        let merged = merge_connected_targets(vec![], vec![]);
        assert!(merged.is_empty());
    }
}
