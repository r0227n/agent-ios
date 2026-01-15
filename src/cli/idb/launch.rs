use crate::companion::CompanionState;
use crate::grpc::{IdbClient, LaunchConfig};
use crate::types::Address;
use std::collections::HashMap;
use tokio::sync::watch;

pub async fn run(
    bundle_id: String,
    app_arguments: Vec<String>,
    udid: Option<String>,
    wait_for_debugger: bool,
    foreground_if_running: bool,
    wait_for: bool,
    pid_file: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Resolve companion by UDID
    let state = CompanionState::default();
    let companion = match udid.as_deref() {
        std::option::Option::Some(u) => state
            .find_by_udid(u)
            .ok_or_else(|| format!("No companion found for UDID: {}", u))?,
        std::option::Option::None => state
            .get_companions()
            .into_iter()
            .next()
            .ok_or("No companions available. Run 'idb_companion' first.")?,
    };
    // ... existing code ...

    // 2. Connect to companion
    let address = companion
        .address()
        .ok_or("Companion has no valid address")?;

    let mut client = match &address {
        Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
        Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
    };

    // 3. Setup stop signal for --wait-for
    let (stop_tx, stop_rx) = watch::channel(false);

    if wait_for {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            stop_tx.send(true).ok();
        });
    }

    // 4. Collect IDB_ env vars
    let env = collect_idb_env();

    // 5. Launch
    let config = LaunchConfig {
        bundle_id,
        app_args: app_arguments,
        env,
        foreground_if_running,
        wait_for_debugger,
    };
    let pid = client.launch(config, wait_for, stop_rx).await?;

    // 6. Write PID file if specified
    if let (Some(p), Some(ref path)) = (pid, &pid_file) {
        std::fs::write(path, format!("{}", p))?;
    }

    Ok(())
}

/// Collect environment variables with IDB_ prefix and strip the prefix
fn collect_idb_env() -> HashMap<String, String> {
    std::env::vars()
        .filter(|(k, _)| k.starts_with("IDB_"))
        .filter_map(|(k, v)| {
            k.strip_prefix("IDB_")
                .map(|stripped| (stripped.to_string(), v))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_idb_env() {
        // Set test environment variables
        std::env::set_var("IDB_TEST_VAR", "test_value");
        std::env::set_var("IDB_ANOTHER", "another_value");
        std::env::set_var("NOT_IDB_VAR", "should_not_appear");

        let env = collect_idb_env();

        assert_eq!(env.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(env.get("ANOTHER"), Some(&"another_value".to_string()));
        assert!(env.get("NOT_IDB_VAR").is_none());
        assert!(env.get("IDB_TEST_VAR").is_none()); // prefix should be stripped

        // Cleanup
        std::env::remove_var("IDB_TEST_VAR");
        std::env::remove_var("IDB_ANOTHER");
        std::env::remove_var("NOT_IDB_VAR");
    }
}
