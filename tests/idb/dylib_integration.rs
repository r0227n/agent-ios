mod common;

use std::process::Command;

/// Test dylib install CLI help output
#[test]
fn test_dylib_install_cli_help() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dylib", "install", "--help"])
        .output()
        .expect("Failed to run dylib install --help");

    assert!(
        output.status.success(),
        "dylib install --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have path argument
    assert!(
        stdout.contains("DYLIB_PATH") || stdout.contains("dylib_path"),
        "Expected dylib_path argument in help output"
    );
    // Should have --json option
    assert!(
        stdout.contains("--json"),
        "Expected --json option in help output"
    );
}

/// Test dylib install help compatibility with Python idb
#[test]
fn test_dylib_install_help_python_compatibility() {
    common::build_agent_mobile();

    // Python idb
    let python_output = Command::new("idb")
        .args(["dylib", "install", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dylib", "install", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test dylib install rejects nonexistent path
#[test]
fn test_dylib_install_nonexistent_path() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_dylib = format!("/tmp/nonexistent_dylib_{}.dylib", std::process::id());

    // Run agent-mobile dylib install
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "dylib",
            "install",
            &nonexistent_dylib,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Should fail for nonexistent path
    assert!(!rust_output.status.success());
}
