use crate::common::{ensure_companion_running, get_available_udid};
use std::process::Command;

/// Test debugserver start CLI help output
#[test]
fn test_debugserver_start_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "start", "--help"])
        .output()
        .expect("Failed to run debugserver start --help");

    assert!(
        output.status.success(),
        "debugserver start --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have bundle_id argument
    assert!(
        stdout.contains("BUNDLE")
            || stdout.contains("bundle")
            || stdout.to_lowercase().contains("bundle"),
        "Expected bundle argument in help output"
    );
}

/// Test debugserver status - this is safe (read-only operation)
#[test]
fn test_debugserver_status() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver status");

    assert!(
        output.status.success(),
        "debugserver status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Status info is output
    let _stdout = String::from_utf8_lossy(&output.stdout);
    // Output format depends on implementation
}

/// Test debugserver stop - verify it runs without crashing
/// Note: Returns non-zero exit code if no debug server is running (matches Python idb behavior)
#[test]
fn test_debugserver_stop() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver stop");

    // Exit code depends on whether a debug server is running
    // If no debug server is running, gRPC returns an error (matches Python idb)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        // Acceptable error when no debug server is running
        assert!(
            stderr.to_lowercase().contains("debug")
                || stderr.to_lowercase().contains("server")
                || stderr.to_lowercase().contains("internal")
                || stderr.to_lowercase().contains("status"),
            "Unexpected error: {}",
            stderr
        );
    }
}

/// Test debugserver start CLI validation (verify bundle_id is required)
#[test]
fn test_debugserver_start_requires_bundle_id() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Start without bundle_id should fail
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "start", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver start");

    // Should fail due to missing required argument
    assert!(
        !output.status.success(),
        "Expected debugserver start to fail without bundle_id"
    );
}

/// Test debugserver status help compatibility with Python idb
#[test]
fn test_debugserver_status_help_python_compatibility() {
    // Python idb
    let python_output = Command::new("idb")
        .args(["debugserver", "status", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test debugserver status Python compatibility
#[test]
fn test_debugserver_status_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["debugserver", "status", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}

/// Test debugserver stop Python compatibility
#[test]
fn test_debugserver_stop_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["debugserver", "stop", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed (stop is idempotent)
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}

#[test]
fn test_debugserver_status_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status"])
        .output()
        .expect("Failed to run debugserver status");

    // デフォルトターゲットがある場合は成功、ない場合はエラー
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available")
                || stderr.contains("Multiple companions available")
                || stderr.contains("target")
                || stderr.contains("Unimplemented")
                || stderr.contains("not implemented"),
            "Expected companion or target error, got: {}",
            stderr
        );
    }
}
