use std::process::Command;

/// Get a valid UDID from idb list-targets (prefers Booted simulator)
fn get_available_udid() -> Option<String> {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(udid) = json.get("udid").and_then(|v| v.as_str()) {
                // Prefer Booted simulator
                if json.get("state").and_then(|v| v.as_str()) == Some("Booted") {
                    return Some(udid.to_string());
                }
            }
        }
    }
    None
}

/// Get installed app bundle ID for testing (Settings app is always available)
fn get_test_bundle_id() -> String {
    "com.apple.Preferences".to_string()
}

#[test]
fn test_launch_basic_execution() {
    let udid = match get_available_udid() {
        Some(u) => u,
        None => {
            eprintln!("Skipping test: no booted simulator available");
            return;
        }
    };

    let bundle_id = get_test_bundle_id();

    // Test that agent-mobile idb launch executes without error
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "launch", "--udid", &udid, &bundle_id])
        .output();

    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                panic!("agent-mobile idb launch failed: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to execute agent-mobile: {}", e);
        }
    }
}

#[test]
fn test_launch_matches_idb_behavior() {
    let udid = match get_available_udid() {
        Some(u) => u,
        None => {
            eprintln!("Skipping test: no booted simulator available");
            return;
        }
    };

    let bundle_id = get_test_bundle_id();

    // Run idb launch
    let idb_result = Command::new("idb")
        .args(["launch", "--udid", &udid, &bundle_id])
        .output();

    // Run agent-mobile idb launch
    let agent_result = Command::new("./target/debug/agent-mobile")
        .args(["idb", "launch", "--udid", &udid, &bundle_id])
        .output();

    match (idb_result, agent_result) {
        (Ok(idb), Ok(agent)) => {
            // Both should succeed
            assert!(
                idb.status.success(),
                "idb launch failed: {}",
                String::from_utf8_lossy(&idb.stderr)
            );
            assert!(
                agent.status.success(),
                "agent-mobile launch failed: {}",
                String::from_utf8_lossy(&agent.stderr)
            );
        }
        (Ok(_), Err(e)) => {
            panic!("agent-mobile failed to execute: {}", e);
        }
        (Err(_), Ok(_)) => {
            eprintln!("Skipping comparison: idb not available");
        }
        (Err(e1), Err(e2)) => {
            panic!("Both commands failed: idb={}, agent={}", e1, e2);
        }
    }
}

#[test]
fn test_launch_with_foreground_flag() {
    let udid = match get_available_udid() {
        Some(u) => u,
        None => {
            eprintln!("Skipping test: no booted simulator available");
            return;
        }
    };

    let bundle_id = get_test_bundle_id();

    // Test --foreground-if-running flag
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "launch",
            "--udid",
            &udid,
            "--foreground-if-running",
            &bundle_id,
        ])
        .output();

    assert!(
        output.is_ok() && output.unwrap().status.success(),
        "launch with --foreground-if-running should succeed"
    );
}

#[test]
fn test_launch_invalid_udid() {
    let bundle_id = get_test_bundle_id();

    // Test with invalid UDID - should fail gracefully
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "launch", "--udid", "INVALID-UDID-12345", &bundle_id])
        .output();

    match output {
        Ok(result) => {
            // Should fail with non-zero exit code
            assert!(
                !result.status.success(),
                "launch with invalid UDID should fail"
            );
        }
        Err(e) => {
            panic!("Command execution failed: {}", e);
        }
    }
}

#[test]
fn test_launch_invalid_bundle_id() {
    let udid = match get_available_udid() {
        Some(u) => u,
        None => {
            eprintln!("Skipping test: no booted simulator available");
            return;
        }
    };

    // Test with invalid bundle ID
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "launch", "--udid", &udid, "com.invalid.notexist.app"])
        .output();

    match output {
        Ok(result) => {
            // Should fail because app doesn't exist
            assert!(
                !result.status.success(),
                "launch with invalid bundle_id should fail"
            );
        }
        Err(e) => {
            panic!("Command execution failed: {}", e);
        }
    }
}
