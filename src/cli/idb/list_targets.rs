use crate::companion::CompanionState;
use crate::grpc::IdbClient;
use crate::simctl;
use crate::types::{
    human_format_target, json_format_target, Address, TargetDescription, TargetType,
};
use std::collections::HashMap;

/// Determine if simulators should be listed based on the filter
fn should_list_simulators(filter: Option<&TargetType>) -> bool {
    filter.map(|f| *f == TargetType::Simulator).unwrap_or(true)
}

pub async fn run(
    only: Option<String>,
    human_output: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse filter
    let filter: Option<TargetType> = only.map(|s| {
        s.parse().unwrap_or_else(|_| {
            panic!("Invalid filter value: {}. Use device, simulator, or mac", s)
        })
    });

    let mut targets: Vec<TargetDescription> = Vec::new();
    let mut connected_udids: HashMap<String, ()> = HashMap::new();

    // Get stored companions from state file and query connected targets
    let state = CompanionState::default();
    let companions = state.get_companions();

    for companion in companions {
        let address = match companion.address() {
            Some(addr) => addr,
            None => continue,
        };

        let result = match &address {
            Address::DomainSocket { path } => IdbClient::connect_uds(path).await,
            Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await,
        };

        let mut client = match result {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Ok(target) = client.describe(false).await {
            connected_udids.insert(target.udid.clone(), ());
            targets.push(target);
        }
    }

    // Get local simulators via simctl (only if filter allows simulators or no filter)
    if should_list_simulators(filter.as_ref()) {
        if let Ok(local_targets) = simctl::list_simulators() {
            for target in local_targets {
                // Skip if already connected via companion
                if connected_udids.contains_key(&target.udid) {
                    continue;
                }
                targets.push(target);
            }
        }
    }

    // Apply filter
    if let Some(ref filter_type) = filter {
        targets.retain(|t| &t.target_type == filter_type);
    }

    // Sort by name (matching Python behavior)
    targets.sort_by(|a, b| a.name.cmp(&b.name));

    // Output results (default: JSON, --human: human-readable)
    let formatter: fn(&TargetDescription) -> String = if human_output {
        human_format_target
    } else {
        json_format_target
    };

    for target in targets {
        println!("{}", formatter(&target));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_list_simulators_no_filter() {
        assert!(should_list_simulators(None));
    }

    #[test]
    fn test_should_list_simulators_with_simulator_filter() {
        assert!(should_list_simulators(Some(&TargetType::Simulator)));
    }

    #[test]
    fn test_should_list_simulators_with_device_filter() {
        assert!(!should_list_simulators(Some(&TargetType::Device)));
    }

    #[test]
    fn test_should_list_simulators_with_mac_filter() {
        assert!(!should_list_simulators(Some(&TargetType::Mac)));
    }
}
