mod common;

use std::process::Command;

/// Test photos-clear CLI help output
#[test]
fn test_photos_clear_cli_help() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "photos-clear", "--help"])
        .output()
        .expect("Failed to run photos-clear --help");

    assert!(
        output.status.success(),
        "photos-clear --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

/// Test photos-clear CLI help
/// Note: Python idb does not have "photos clear" command in released versions (as of 1.1.7).
/// The feature was added to the idb repository in November 2025 but not yet released to PyPI.
/// This test validates agent-mobile's implementation only.
#[test]
fn test_photos_clear_help_agent_mobile_only() {
    common::build_agent_mobile();

    // agent-mobile (Python idb does not have "photos clear" in released versions)
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "photos-clear", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    assert!(rust_output.status.success(), "agent-mobile help failed");
}
