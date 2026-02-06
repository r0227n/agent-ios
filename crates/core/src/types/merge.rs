//! Target merging utilities.

use super::DeviceInfo;
use std::collections::HashMap;

/// Merge local targets with connected targets.
///
/// - When the same UDID exists in both, prefer the connected target (with companion_info)
/// - Add any remote targets that aren't in local targets
pub fn merge_connected_targets(
    local_targets: Vec<DeviceInfo>,
    connected_targets: Vec<DeviceInfo>,
) -> Vec<DeviceInfo> {
    let connected_map: HashMap<String, DeviceInfo> = connected_targets
        .into_iter()
        .map(|t| (t.udid.clone(), t))
        .collect();

    let mut targets: HashMap<String, DeviceInfo> = HashMap::new();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, CompanionInfo, TargetType};

    fn make_target(name: &str, udid: &str, with_companion: bool) -> DeviceInfo {
        DeviceInfo {
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
                        path: format!("/tmp/{}_companion.sock", udid),
                    },
                })
            } else {
                None
            },
        }
    }

    #[test]
    fn test_merge_prefers_connected() {
        let local = vec![make_target("iPhone 14", "UDID-1", false)];
        let connected = vec![make_target("iPhone 14", "UDID-1", true)];

        let merged = merge_connected_targets(local, connected);

        assert_eq!(merged.len(), 1);
        assert!(merged[0].companion_info.is_some());
    }

    #[test]
    fn test_merge_adds_remote_only() {
        let local = vec![make_target("iPhone 14", "UDID-1", false)];
        let connected = vec![make_target("Remote Device", "UDID-2", true)];

        let merged = merge_connected_targets(local, connected);

        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_keeps_local_without_connected() {
        let local = vec![make_target("iPhone 14", "UDID-1", false)];
        let connected = vec![];

        let merged = merge_connected_targets(local, connected);

        assert_eq!(merged.len(), 1);
        assert!(merged[0].companion_info.is_none());
    }

    #[test]
    fn test_merge_empty_inputs() {
        let merged = merge_connected_targets(vec![], vec![]);
        assert!(merged.is_empty());
    }
}
