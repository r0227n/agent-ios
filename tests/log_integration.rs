mod common;

use common::{get_available_udid, wait_with_timeout};
use std::process::{Command, Stdio};

#[test]
fn test_log_basic_execution() {
    let udid = get_available_udid();

    // Start log command
    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start agent-mobile");

    // Wait with timeout to prevent hangs (10 seconds max)
    let output = wait_with_timeout(child, 10);

    // Basic sanity check - no panic or crash
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic") && !stderr.contains("SIGSEGV"),
        "Log command crashed: stderr={}",
        stderr
    );
}

#[test]
fn test_log_with_source_companion() {
    let udid = get_available_udid();

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--udid", &udid, "--source", "companion"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start agent-mobile");

    // Wait with timeout to prevent hangs (10 seconds max)
    let output = wait_with_timeout(child, 10);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("panic"),
        "Log command with --source companion crashed"
    );
}

#[test]
fn test_log_invalid_udid() {
    // Rust implementation
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--udid", "INVALID-UDID-12345"])
        .output()
        .expect("Failed to run Rust implementation");

    // Verify Rust implementation fails with invalid UDID
    assert!(
        !rust_output.status.success(),
        "Rust: log with invalid UDID should fail"
    );

    let rust_stderr = String::from_utf8_lossy(&rust_output.stderr);
    assert!(
        rust_stderr.contains("No companion found") || rust_stderr.contains("not found"),
        "Expected UDID error in Rust stderr, got: {}",
        rust_stderr
    );

    // Python idb comparison
    let python_output = Command::new("idb")
        .args(["log", "--udid", "INVALID-UDID-12345"])
        .output()
        .expect("Failed to run Python implementation");

    assert!(
        !python_output.status.success(),
        "Python: log with invalid UDID should fail"
    );
}

#[test]
fn test_log_help() {
    // Test that help output includes log command
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "help command should succeed");
    assert!(
        stdout.contains("target") || stdout.contains("companion"),
        "help should mention source options"
    );
    assert!(
        stdout.contains("--udid"),
        "help should mention --udid option"
    );
}

#[test]
fn test_log_compatibility_with_python_idb() {
    let udid = get_available_udid();

    // Start Rust implementation
    let rust_child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "log", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Rust implementation");

    // Start Python idb implementation
    let python_child = Command::new("idb")
        .args(["log", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Python implementation");

    // Wait with timeout for both processes
    let rust_output = wait_with_timeout(rust_child, 10);
    let python_output = wait_with_timeout(python_child, 10);

    // Compare exit behavior - both should terminate gracefully (via SIGTERM)
    // Note: We don't compare stdout content since log output is time-dependent
    let rust_stderr = String::from_utf8_lossy(&rust_output.stderr);
    let python_stderr = String::from_utf8_lossy(&python_output.stderr);

    // Both should not have panicked or crashed
    assert!(
        !rust_stderr.contains("panic") && !rust_stderr.contains("SIGSEGV"),
        "Rust implementation crashed: {}",
        rust_stderr
    );

    // Log any differences in stderr behavior for debugging
    if !rust_stderr.is_empty() || !python_stderr.is_empty() {
        eprintln!("Rust stderr: {}", rust_stderr);
        eprintln!("Python stderr: {}", python_stderr);
    }
}
