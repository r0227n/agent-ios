mod common;

use common::{ensure_companion_running, get_available_udid, get_test_bundle_id};
use std::process::Command;

#[test]
fn test_launch_basic_execution() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
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
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    // Run idb launch
    let idb_result = Command::new("idb")
        .args(["launch", "--udid", &udid, &bundle_id])
        .output()
        .expect("Failed to execute Python idb - ensure idb is installed and in PATH");

    // Run agent-mobile idb launch
    let agent_result = Command::new("./target/debug/agent-mobile")
        .args(["idb", "launch", "--udid", &udid, &bundle_id])
        .output()
        .expect("Failed to execute agent-mobile command");

    // Both should succeed
    assert!(
        idb_result.status.success(),
        "Python idb launch failed: {}",
        String::from_utf8_lossy(&idb_result.stderr)
    );
    assert!(
        agent_result.status.success(),
        "agent-mobile launch failed: {}",
        String::from_utf8_lossy(&agent_result.stderr)
    );
}

#[test]
fn test_launch_with_foreground_flag() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
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
    let udid = get_available_udid();

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
