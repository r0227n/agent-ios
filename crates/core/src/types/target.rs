//! Target and address type definitions.

use serde::{Deserialize, Serialize};
use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_address_display() {
        let tcp = Address::Tcp {
            host: "localhost".to_string(),
            port: 10882,
        };
        assert_eq!(format!("{}", tcp), "localhost:10882");

        let uds = Address::DomainSocket {
            path: "/tmp/idb.sock".to_string(),
        };
        assert_eq!(format!("{}", uds), "/tmp/idb.sock");
    }
}
