//! Target and address type definitions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of iOS target (simulator, physical device, or Mac)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TargetType {
    /// An iOS simulator managed by CoreSimulator.
    Simulator,
    /// A physical mobile device.
    Device,
    /// A locally addressable Mac host target.
    Mac,
}

impl TargetType {
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

/// Network connection address
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Address {
    /// TCP connection (host:port)
    Tcp {
        /// Host name or IP address of the remote endpoint.
        host: String,
        /// TCP port of the remote endpoint.
        port: u16,
    },
    /// Unix domain socket connection
    DomainSocket {
        /// Filesystem path to the Unix domain socket.
        path: String,
    },
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Tcp { host, port } => write!(f, "{}:{}", host, port),
            Address::DomainSocket { path } => write!(f, "{}", path),
        }
    }
}

/// Connection daemon info (legacy, currently unused)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionInfo {
    /// Target device UDID.
    pub udid: String,
    /// Whether the connection is local
    pub is_local: bool,
    /// Daemon process ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Connection address (TCP or Unix socket).
    pub address: Address,
}

/// Device information (iOS target - device or simulator).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device display name.
    pub name: String,
    /// Unique device identifier.
    pub udid: String,
    /// Current device state (e.g., "Booted", "Shutdown").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Type of target (simulator, device, or mac).
    pub target_type: TargetType,
    /// Operating system version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    /// CPU architecture (e.g., "arm64", "x86_64").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    /// Connection info (currently always None)
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
    fn test_address_display() {
        let tcp = Address::Tcp {
            host: "localhost".to_string(),
            port: 10882,
        };
        assert_eq!(format!("{}", tcp), "localhost:10882");

        let uds = Address::DomainSocket {
            path: "/tmp/test.sock".to_string(),
        };
        assert_eq!(format!("{}", uds), "/tmp/test.sock");
    }
}
