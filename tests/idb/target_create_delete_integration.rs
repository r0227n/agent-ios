use std::process::Command;

/// Test target create CLI help output
#[test]
fn test_target_create_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "create", "--help"])
        .output()
        .expect("Failed to run target create --help");

    assert!(
        output.status.success(),
        "target create --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have name, device-type, and os-version arguments
    assert!(
        stdout.contains("NAME") || stdout.contains("name"),
        "Expected name argument in help output"
    );
}

/// Test target clone CLI help output
#[test]
fn test_target_clone_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "clone", "--help"])
        .output()
        .expect("Failed to run target clone --help");

    assert!(
        output.status.success(),
        "target clone --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have source udid and new name arguments
    assert!(
        stdout.contains("UDID") || stdout.contains("udid") || stdout.contains("source"),
        "Expected source udid argument in help output"
    );
}

/// Test target delete CLI help output
#[test]
fn test_target_delete_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "delete", "--help"])
        .output()
        .expect("Failed to run target delete --help");

    assert!(
        output.status.success(),
        "target delete --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have --all flag or udid argument
    assert!(
        stdout.contains("--all") || stdout.contains("UDID") || stdout.contains("udid"),
        "Expected --all flag or udid argument in help output"
    );
}

/// Test target create help compatibility with Python idb
/// Note: Python idb uses "idb create" (top-level), while agent-mobile uses "idb target create"
#[test]
fn test_target_create_help_python_compatibility() {
    // Python idb uses top-level "create" command
    let python_output = Command::new("idb")
        .args(["create", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses nested "target create" command
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "create", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test target clone help compatibility with Python idb
/// Note: Python idb uses "idb clone" (top-level), while agent-mobile uses "idb target clone"
#[test]
fn test_target_clone_help_python_compatibility() {
    // Python idb uses top-level "clone" command
    let python_output = Command::new("idb")
        .args(["clone", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses nested "target clone" command
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "clone", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test target delete help compatibility with Python idb
/// Note: Python idb uses "idb delete" (top-level), while agent-mobile uses "idb target delete"
#[test]
fn test_target_delete_help_python_compatibility() {
    // Python idb uses top-level "delete" command
    let python_output = Command::new("idb")
        .args(["delete", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses nested "target delete" command
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "delete", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test target delete --all CLI validation (verify flag exists)
#[test]
fn test_target_delete_all_flag_exists() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "delete", "--help"])
        .output()
        .expect("Failed to run target delete --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--all"),
        "Expected --all flag in help output"
    );
}
