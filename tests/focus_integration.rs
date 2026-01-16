mod common;

use common::get_available_udid;
use std::process::Command;

#[test]
fn test_focus_with_udid() {
    let udid = get_available_udid();

    // Test Rust implementation
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "focus", "--udid", &udid])
        .output()
        .expect("Failed to run focus command");

    assert!(
        rust_output.status.success(),
        "Rust focus command failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );

    // Verify no output (focus is silent on success)
    assert!(
        rust_output.stdout.is_empty(),
        "Expected no stdout output, got: {}",
        String::from_utf8_lossy(&rust_output.stdout)
    );

    // Test Python idb for comparison
    let python_output = Command::new("idb")
        .args(["focus", "--udid", &udid])
        .output()
        .expect("Failed to run Python idb focus command");

    assert!(
        python_output.status.success(),
        "Python idb focus command failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );

    // Both should have no stdout output
    assert!(python_output.stdout.is_empty());
}

#[test]
fn test_focus_without_udid() {
    // Test with default target (first available companion)
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "focus"])
        .output()
        .expect("Failed to run focus command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should fail gracefully if no companions available
        assert!(
            stderr.contains("No companions available"),
            "Expected companion error, got: {}",
            stderr
        );
    } else {
        // If succeeded, verify no output
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn test_focus_invalid_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "focus", "--udid", "INVALID_UDID_12345"])
        .output()
        .expect("Failed to run focus command");

    assert!(!output.status.success(), "Should fail with invalid UDID");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No companion found for UDID"),
        "Expected UDID error, got: {}",
        stderr
    );
}

#[test]
fn test_focus_compatibility_with_python_idb() {
    let udid = get_available_udid();

    // Run both implementations
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "focus", "--udid", &udid])
        .output()
        .expect("Failed to run Rust implementation");

    let python_output = Command::new("idb")
        .args(["focus", "--udid", &udid])
        .output()
        .expect("Failed to run Python implementation");

    // Both should succeed
    assert_eq!(
        rust_output.status.success(),
        python_output.status.success(),
        "Rust and Python implementations have different exit codes"
    );

    // Both should produce identical output (empty)
    assert_eq!(
        rust_output.stdout, python_output.stdout,
        "Rust and Python implementations produce different stdout"
    );

    // Compare stderr behavior (both should be empty or similar)
    let rust_stderr = String::from_utf8_lossy(&rust_output.stderr);
    let python_stderr = String::from_utf8_lossy(&python_output.stderr);

    if !rust_stderr.is_empty() || !python_stderr.is_empty() {
        eprintln!("Rust stderr: {}", rust_stderr);
        eprintln!("Python stderr: {}", python_stderr);
    }
}
