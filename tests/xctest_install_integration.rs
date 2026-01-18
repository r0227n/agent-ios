mod common;

use std::process::Command;

/// Test xctest-install CLI help output
#[test]
fn test_xctest_install_cli_help() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-install", "--help"])
        .output()
        .expect("Failed to run xctest-install --help");

    assert!(
        output.status.success(),
        "xctest-install --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have path argument
    assert!(
        stdout.contains("PATH")
            || stdout.contains("path")
            || stdout.contains("BUNDLE")
            || stdout.contains("bundle"),
        "Expected path argument in help output"
    );
}

/// Test xctest-install has --json flag
#[test]
fn test_xctest_install_has_json_flag() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-install", "--help"])
        .output()
        .expect("Failed to run xctest-install --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--json"),
        "Expected --json flag in help output"
    );
}

/// Test xctest-install rejects nonexistent bundle
#[test]
fn test_xctest_install_nonexistent_bundle() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_bundle = format!("/tmp/nonexistent_test_bundle_{}.xctest", std::process::id());

    // Run Python idb xctest install
    let python_output = Command::new("idb")
        .args(["xctest", "install", &nonexistent_bundle, "--udid", &udid])
        .output()
        .expect("Failed to run Python idb");

    // Run agent-mobile xctest-install
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "xctest-install",
            &nonexistent_bundle,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should fail
    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());
}

/// Test xctest-install help compatibility with Python idb
#[test]
fn test_xctest_install_help_python_compatibility() {
    common::build_agent_mobile();

    // Python idb
    let python_output = Command::new("idb")
        .args(["xctest", "install", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-install", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}
