use std::process::Command;

/// Integration test for mkdir command
#[test]
#[ignore] // Run with: cargo test --test mkdir_integration -- --ignored
fn test_mkdir_creates_directory() {
    // Get UDID of first booted simulator
    let list_targets_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-targets"])
        .output()
        .expect("Failed to run list-targets");

    let targets_str = String::from_utf8_lossy(&list_targets_output.stdout);
    let udid = targets_str
        .lines()
        .find(|line| line.contains("\"state\":\"Booted\""))
        .and_then(|line| {
            line.split("\"udid\":\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
        })
        .expect("No booted simulator found. Please boot a simulator first.");

    // Create a test directory in root container
    let test_path = format!("/tmp/test_mkdir_{}", std::process::id());

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "mkdir", &test_path, "--root", "--udid", udid])
        .output()
        .expect("Failed to run agent-mobile mkdir");

    // Should not error
    if !output.status.success() {
        panic!("mkdir failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    // No output on success (matching Python idb behavior)
    assert!(
        output.stdout.is_empty(),
        "Expected no output on success, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

/// Test that mkdir works without UDID when only one target is available
#[test]
#[ignore]
fn test_mkdir_without_udid() {
    let test_path = format!("/tmp/test_mkdir_no_udid_{}", std::process::id());

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "mkdir", &test_path, "--root"])
        .output()
        .expect("Failed to run agent-mobile");

    // Should not error if at least one simulator is booted
    if !output.status.success() {
        panic!("mkdir failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}

/// Test that mkdir fails with appropriate error for invalid container
#[test]
#[ignore]
fn test_mkdir_invalid_bundle_id() {
    let test_path = "/tmp/test_invalid";

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "mkdir",
            test_path,
            "--bundle-id",
            "com.nonexistent.app",
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Should error
    assert!(
        !output.status.success(),
        "Expected error for invalid bundle ID"
    );
}

/// Compare mkdir behavior with Python idb
#[test]
#[ignore]
fn test_mkdir_compatibility_with_python_idb() {
    let list_targets_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-targets"])
        .output()
        .expect("Failed to run list-targets");

    let targets_str = String::from_utf8_lossy(&list_targets_output.stdout);
    let udid = targets_str
        .lines()
        .find(|line| line.contains("\"state\":\"Booted\""))
        .and_then(|line| {
            line.split("\"udid\":\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
        })
        .expect("No booted simulator found");

    let test_path1 = format!("/tmp/test_python_mkdir1_{}", std::process::id());
    let test_path2 = format!("/tmp/test_python_mkdir2_{}", std::process::id());

    // Run Python idb
    let python_output = Command::new("idb")
        .args(["file", "mkdir", &test_path1, "--root", "--udid", udid])
        .output()
        .expect("Failed to run Python idb");

    // Run Rust implementation
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "mkdir", &test_path2, "--root", "--udid", udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed with no output
    assert_eq!(python_output.status.success(), rust_output.status.success());
    assert_eq!(
        python_output.stdout.is_empty(),
        rust_output.stdout.is_empty()
    );
}
