mod common;

use common::get_available_udid;
use std::process::Command;

#[test]
fn test_uninstall_invalid_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "uninstall",
            "--udid",
            "INVALID-UDID",
            "com.example.app",
        ])
        .output()
        .expect("Failed to run uninstall command");

    assert!(
        !output.status.success(),
        "Command should fail with invalid UDID"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No companion found for UDID"),
        "Error message should mention companion not found, got: {}",
        stderr
    );
}

#[test]
fn test_uninstall_invalid_bundle_id() {
    let udid = get_available_udid();
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "uninstall",
            "--udid",
            &udid,
            "com.nonexistent.app.that.does.not.exist",
        ])
        .output()
        .expect("Failed to run uninstall command");

    // Invalid bundle ID should result in gRPC error
    assert!(
        !output.status.success(),
        "Command should fail with invalid bundle ID"
    );
}

#[test]
fn test_uninstall_without_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "uninstall",
            "com.nonexistent.app.that.does.not.exist",
        ])
        .output()
        .expect("Failed to run uninstall command");

    // When companion is available, command should execute (may fail with gRPC error)
    // When no companion, should get appropriate error
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either gRPC error or "No companions available" error is acceptable
    assert!(
        !output.status.success()
            && (stderr.contains("No companions available")
                || stderr.contains("error")
                || stderr.contains("Error")),
        "Should fail with appropriate error, got: {}",
        stderr
    );
}

#[test]
fn test_uninstall_with_udid() {
    let udid = get_available_udid();
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "uninstall",
            "--udid",
            &udid,
            "com.nonexistent.app.that.does.not.exist",
        ])
        .output()
        .expect("Failed to run uninstall command");

    // Should fail due to invalid bundle ID, but UDID should be correctly processed
    assert!(
        !output.status.success(),
        "Command should fail with invalid bundle ID"
    );

    // Should NOT contain "No companion found for UDID" error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("No companion found for UDID"),
        "Should not have UDID error since UDID is valid, got: {}",
        stderr
    );
}

#[test]
fn test_uninstall_compatibility_with_python_idb() {
    let udid = get_available_udid();
    let bundle_id = "com.nonexistent.app.that.does.not.exist";

    // Run Rust implementation
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "uninstall", "--udid", &udid, bundle_id])
        .output()
        .expect("Failed to run Rust implementation");

    // Run Python implementation
    let python_output = Command::new("idb")
        .args(["uninstall", "--udid", &udid, bundle_id])
        .output()
        .expect("Failed to run Python implementation");

    // Both should have same exit status (both should fail)
    assert_eq!(
        rust_output.status.success(),
        python_output.status.success(),
        "Rust and Python implementations have different exit codes"
    );

    // Both should produce no stdout output (errors go to stderr)
    assert_eq!(
        rust_output.stdout, python_output.stdout,
        "Rust and Python implementations produce different stdout"
    );

    // Both stdout should be empty
    assert!(
        rust_output.stdout.is_empty(),
        "Rust implementation should have no stdout output"
    );
    assert!(
        python_output.stdout.is_empty(),
        "Python implementation should have no stdout output"
    );
}
