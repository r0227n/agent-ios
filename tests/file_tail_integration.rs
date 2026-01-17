mod common;

use std::fs;
use std::process::Command;

/// Integration tests for file tail command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
#[ignore] // Run with: cargo test --test file_tail_integration -- --ignored
fn test_tail_existing_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let device_path = format!("/tmp/test_tail_{}", pid);

    // Create test file on device with content
    let test_content = b"line 1\nline 2\nline 3\n";
    let temp_local = format!("/tmp/test_tail_local_{}", pid);
    fs::write(&temp_local, test_content).expect("Failed to create local test file");
    let _push =
        common::run_idb_file_command(&["push", &temp_local, &device_path, "--udid", &udid]);

    // Tail with Python idb (spawn and terminate after brief period)
    let python_child = Command::new("idb")
        .args(["file", "tail", &device_path, "--udid", &udid])
        .spawn()
        .expect("Failed to spawn Python idb tail");

    let python_output = common::wait_with_timeout(python_child, 5);

    // Tail with agent-mobile (spawn and terminate after brief period)
    let rust_child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "file", "tail", &device_path, "--udid", &udid])
        .spawn()
        .expect("Failed to spawn agent-mobile tail");

    let rust_output = common::wait_with_timeout(rust_child, 5);

    // Both should have received SIGTERM and exited gracefully
    // Exit codes should match (both should be 0 or signal-based)
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ for tail with SIGTERM"
    );

    // Both should have outputted the file content
    let python_stdout = String::from_utf8_lossy(&python_output.stdout);
    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

    // Check that both contain the expected content
    assert!(
        python_stdout.contains("line 1") || python_stdout.contains("line1"),
        "Python tail output doesn't contain expected content"
    );
    assert!(
        rust_stdout.contains("line 1") || rust_stdout.contains("line1"),
        "Rust tail output doesn't contain expected content"
    );

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &device_path, "--udid", &udid]);
    let _ = fs::remove_file(&temp_local);
}

#[test]
#[ignore]
fn test_tail_with_bundle_id_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);
    let bundle_id = common::get_test_bundle_id();

    let pid = std::process::id();
    let device_path = format!("/tmp/test_tail_bundle_{}", pid);

    // Create test file in app container
    let test_content = b"app log line 1\napp log line 2\n";
    let temp_local = format!("/tmp/test_tail_bundle_local_{}", pid);
    fs::write(&temp_local, test_content).expect("Failed to create local test file");
    let _push = common::run_idb_file_command(&[
        "push",
        &temp_local,
        &device_path,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Tail with Python idb
    let python_child = Command::new("idb")
        .args([
            "file",
            "tail",
            &device_path,
            "--bundle-id",
            &bundle_id,
            "--udid",
            &udid,
        ])
        .spawn()
        .expect("Failed to spawn Python idb tail");

    let python_output = common::wait_with_timeout(python_child, 5);

    // Tail with agent-mobile
    let rust_child = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "file",
            "tail",
            &device_path,
            "--bundle-id",
            &bundle_id,
            "--udid",
            &udid,
        ])
        .spawn()
        .expect("Failed to spawn agent-mobile tail");

    let rust_output = common::wait_with_timeout(rust_child, 5);

    // Both should have handled SIGTERM similarly
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ for tail with bundle-id"
    );

    // Cleanup
    let _ = common::run_idb_file_command(&[
        "rm",
        &device_path,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let _ = fs::remove_file(&temp_local);
}

#[test]
#[ignore]
fn test_tail_nonexistent_file_error() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_path = "/tmp/this_file_does_not_exist_12345";

    // Both should fail immediately
    let python_output =
        common::run_idb_file_command(&["tail", nonexistent_path, "--udid", &udid]);
    let rust_output =
        common::run_agent_mobile_file_command(&["tail", nonexistent_path, "--udid", &udid]);

    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_tail_empty_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let device_path = format!("/tmp/test_tail_empty_{}", pid);

    // Create empty file on device
    let temp_local = format!("/tmp/test_tail_empty_local_{}", pid);
    fs::write(&temp_local, b"").expect("Failed to create local test file");
    let _push =
        common::run_idb_file_command(&["push", &temp_local, &device_path, "--udid", &udid]);

    // Tail with Python idb
    let python_child = Command::new("idb")
        .args(["file", "tail", &device_path, "--udid", &udid])
        .spawn()
        .expect("Failed to spawn Python idb tail");

    let python_output = common::wait_with_timeout(python_child, 5);

    // Tail with agent-mobile
    let rust_child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "file", "tail", &device_path, "--udid", &udid])
        .spawn()
        .expect("Failed to spawn agent-mobile tail");

    let rust_output = common::wait_with_timeout(rust_child, 5);

    // Both should handle empty file gracefully
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ for empty file tail"
    );

    // Both should have empty or minimal output
    assert!(
        python_output.stdout.is_empty() || python_output.stdout.len() < 10,
        "Expected empty output from Python tail"
    );
    assert!(
        rust_output.stdout.is_empty() || rust_output.stdout.len() < 10,
        "Expected empty output from Rust tail"
    );

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &device_path, "--udid", &udid]);
    let _ = fs::remove_file(&temp_local);
}

#[test]
#[ignore]
fn test_tail_signal_handling() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let device_path = format!("/tmp/test_tail_signal_{}", pid);

    // Create test file
    let test_content = b"test content\n";
    let temp_local = format!("/tmp/test_tail_signal_local_{}", pid);
    fs::write(&temp_local, test_content).expect("Failed to create local test file");
    let _push =
        common::run_idb_file_command(&["push", &temp_local, &device_path, "--udid", &udid]);

    // Test that both implementations handle SIGTERM gracefully
    let python_child = Command::new("idb")
        .args(["file", "tail", &device_path, "--udid", &udid])
        .spawn()
        .expect("Failed to spawn Python idb tail");

    let python_output = common::wait_with_timeout(python_child, 5);

    let rust_child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "file", "tail", &device_path, "--udid", &udid])
        .spawn()
        .expect("Failed to spawn agent-mobile tail");

    let rust_output = common::wait_with_timeout(rust_child, 5);

    // Both should exit with the same status after receiving SIGTERM
    // (may be 0 or 143/SIGTERM depending on signal handling)
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Signal handling differs between implementations"
    );

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &device_path, "--udid", &udid]);
    let _ = fs::remove_file(&temp_local);
}
