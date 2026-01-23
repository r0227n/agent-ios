use agent_mobile_core::{
    human_format_target, json_format_target, merge_connected_targets, Address, TargetDescription,
    TargetType,
};
use agent_mobile_platform_ios::companion::{CompanionLister, CompanionState};
use agent_mobile_platform_ios::grpc::IdbClient;

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

    // 1. Get local targets from idb_companion --list 1
    let local_targets = match CompanionLister::new() {
        Ok(lister) => lister.list_targets(filter).unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // 2. Get connected targets from state file companions
    let connected_targets = get_connected_targets().await;

    // 3. Merge targets (connected takes priority for same UDID)
    let mut targets = merge_connected_targets(local_targets, connected_targets);

    // Apply filter (for connected targets that might not have been filtered)
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

/// Get connected targets by querying companions from state file
async fn get_connected_targets() -> Vec<TargetDescription> {
    let state = CompanionState::default();
    let companions = state.get_companions();
    let mut targets = Vec::new();

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
            targets.push(target);
        }
    }

    targets
}
