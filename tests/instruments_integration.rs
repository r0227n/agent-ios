mod common;

use std::process::Command;

/// Test instruments CLI help output
#[test]
fn test_instruments_cli_help() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "instruments", "--help"])
        .output()
        .expect("Failed to run instruments --help");

    assert!(
        output.status.success(),
        "instruments --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have --template option (required)
    assert!(
        stdout.contains("--template"),
        "Expected --template option in help output"
    );
    // Should have --app-bundle-id option
    assert!(
        stdout.contains("--app-bundle-id"),
        "Expected --app-bundle-id option in help output"
    );
    // Should have --output option
    assert!(
        stdout.contains("--output"),
        "Expected --output option in help output"
    );
    // Should have --operation-duration option
    assert!(
        stdout.contains("--operation-duration"),
        "Expected --operation-duration option in help output"
    );
}

/// Test instruments requires --template
#[test]
fn test_instruments_requires_template() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "instruments"])
        .output()
        .expect("Failed to run instruments");

    // Should fail without --template
    assert!(
        !output.status.success(),
        "instruments should fail without --template"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--template") || stderr.contains("required"),
        "Error should mention --template is required"
    );
}

/// Test instruments help compatibility with Python idb
#[test]
fn test_instruments_help_python_compatibility() {
    common::build_agent_mobile();

    // Python idb
    let python_output = Command::new("idb")
        .args(["instruments", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "instruments", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");

    // Both should mention key options
    let python_stdout = String::from_utf8_lossy(&python_output.stdout);
    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

    assert!(
        python_stdout.contains("template") && rust_stdout.contains("template"),
        "Both should mention template"
    );
}
