use crate::cli::helpers::{setup_ctrl_c_handler, with_client, CommandResult};
use crate::grpc::LaunchConfig;
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
) -> CommandResult {
    // Setup stop signal for --wait-for
    let stop_rx = if wait_for {
        setup_ctrl_c_handler()
    } else {
        let (_tx, rx) = watch::channel(false);
        rx
    };

    // Collect IDB_ env vars
    let env = collect_idb_env();

    with_client(udid.as_deref(), |mut client| async move {
        // Launch
        let config = LaunchConfig {
            bundle_id,
            app_args: app_arguments,
            env,
            foreground_if_running,
            wait_for_debugger,
        };
        let pid = client.launch(config, wait_for, stop_rx).await?;

        // Write PID file if specified
        if let (Some(p), Some(ref path)) = (pid, &pid_file) {
            std::fs::write(path, format!("{}", p))?;
        }

        Ok(())
    })
    .await
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
        assert!(!env.contains_key("NOT_IDB_VAR"));
        assert!(!env.contains_key("IDB_TEST_VAR")); // prefix should be stripped

        // Cleanup
        std::env::remove_var("IDB_TEST_VAR");
        std::env::remove_var("IDB_ANOTHER");
        std::env::remove_var("NOT_IDB_VAR");
    }
}
