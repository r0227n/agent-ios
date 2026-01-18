mod common;

use std::process::Command;

/// Test dsym install CLI help output
#[test]
fn test_dsym_install_cli_help() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dsym", "install", "--help"])
        .output()
        .expect("Failed to run dsym install --help");

    assert!(
        output.status.success(),
        "dsym install --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have path argument
    assert!(
        stdout.contains("DSYM_PATH") || stdout.contains("dsym_path"),
        "Expected dsym_path argument in help output"
    );
    // Should have --bundle-id option
    assert!(
        stdout.contains("--bundle-id"),
        "Expected --bundle-id option in help output"
    );
    // Should have --json option
    assert!(
        stdout.contains("--json"),
        "Expected --json option in help output"
    );
}

/// Test dsym install help compatibility with Python idb
#[test]
fn test_dsym_install_help_python_compatibility() {
    common::build_agent_mobile();

    // Python idb
    let python_output = Command::new("idb")
        .args(["dsym", "install", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dsym", "install", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test dsym install rejects nonexistent path
#[test]
fn test_dsym_install_nonexistent_path() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_dsym = format!("/tmp/nonexistent_dsym_{}.dSYM", std::process::id());

    // Run agent-mobile dsym install
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dsym", "install", &nonexistent_dsym, "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Should fail for nonexistent path
    assert!(!rust_output.status.success());
}
