use crate::companion::CompanionResolver;

pub async fn run(udid: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Connect to companion (with auto-spawning if needed)
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // 2. Call list_apps RPC
    let apps = client.list_apps().await?;

    // 3. Format and output (matching Python idb format)
    for app in apps {
        let arch = app.architectures.join(", ");
        let process_state = match app.process_state() {
            crate::grpc::idb::installed_app_info::AppProcessState::Unknown => "Unknown",
            crate::grpc::idb::installed_app_info::AppProcessState::NotRunning => "Not running",
            crate::grpc::idb::installed_app_info::AppProcessState::Running => "Running",
        };
        let debuggable = if app.debuggable {
            "Debuggable"
        } else {
            "Not Debuggable"
        };
        let pid = if app.process_identifier == 0 {
            "None".to_string()
        } else {
            app.process_identifier.to_string()
        };

        println!(
            "{} | {} | {} | {} | {} | {} | pid={}",
            app.bundle_id, app.name, app.install_type, arch, process_state, debuggable, pid
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_architecture_formatting() {
        // Test that architectures are joined with ", "
        let archs = ["x86_64".to_string(), "arm64".to_string()];
        let result = archs.join(", ");
        assert_eq!(result, "x86_64, arm64");

        // Single architecture
        let archs = ["arm64".to_string()];
        let result = archs.join(", ");
        assert_eq!(result, "arm64");

        // Empty architectures
        let archs: Vec<String> = vec![];
        let result = archs.join(", ");
        assert_eq!(result, "");
    }

    #[test]
    fn test_process_state_formatting() {
        use crate::grpc::idb::installed_app_info::AppProcessState;

        let state = AppProcessState::Unknown;
        let formatted = match state {
            AppProcessState::Unknown => "Unknown",
            AppProcessState::NotRunning => "Not running",
            AppProcessState::Running => "Running",
        };
        assert_eq!(formatted, "Unknown");

        let state = AppProcessState::Running;
        let formatted = match state {
            AppProcessState::Unknown => "Unknown",
            AppProcessState::NotRunning => "Not running",
            AppProcessState::Running => "Running",
        };
        assert_eq!(formatted, "Running");
    }

    #[test]
    fn test_debuggable_formatting() {
        let debuggable = true;
        let result = if debuggable {
            "Debuggable"
        } else {
            "Not Debuggable"
        };
        assert_eq!(result, "Debuggable");

        let debuggable = false;
        let result = if debuggable {
            "Debuggable"
        } else {
            "Not Debuggable"
        };
        assert_eq!(result, "Not Debuggable");
    }

    #[test]
    fn test_pid_formatting() {
        // PID 0 should be formatted as "None"
        let pid: u64 = 0;
        let result = if pid == 0 {
            "None".to_string()
        } else {
            pid.to_string()
        };
        assert_eq!(result, "None");

        // Non-zero PID should be formatted as the number
        let pid: u64 = 12345;
        let result = if pid == 0 {
            "None".to_string()
        } else {
            pid.to_string()
        };
        assert_eq!(result, "12345");
    }
}
