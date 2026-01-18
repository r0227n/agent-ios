mod common;

use std::process::Command;

/// Test xctrace record CLI help output
#[test]
fn test_xctrace_record_cli_help() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctrace", "record", "--help"])
        .output()
        .expect("Failed to run xctrace record --help");

    assert!(
        output.status.success(),
        "xctrace record --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have --template option (required)
    assert!(
        stdout.contains("--template"),
        "Expected --template option in help output"
    );
    // Should have target options
    assert!(
        stdout.contains("--all-processes"),
        "Expected --all-processes option in help output"
    );
    assert!(
        stdout.contains("--attach"),
        "Expected --attach option in help output"
    );
    assert!(
        stdout.contains("--launch"),
        "Expected --launch option in help output"
    );
    // Should have --time-limit option
    assert!(
        stdout.contains("--time-limit"),
        "Expected --time-limit option in help output"
    );
}

/// Test xctrace record requires --template
#[test]
fn test_xctrace_record_requires_template() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctrace", "record", "--all-processes"])
        .output()
        .expect("Failed to run xctrace record");

    // Should fail without --template
    assert!(
        !output.status.success(),
        "xctrace record should fail without --template"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--template") || stderr.contains("required"),
        "Error should mention --template is required"
    );
}

/// Test xctrace record requires target option
#[test]
fn test_xctrace_record_requires_target() {
    common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctrace", "record", "--template", "Time Profiler"])
        .output()
        .expect("Failed to run xctrace record");

    // Should fail without target option
    assert!(
        !output.status.success(),
        "xctrace record should fail without target option"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("all-processes")
            || stderr.contains("attach")
            || stderr.contains("launch")
            || stderr.contains("must be specified"),
        "Error should mention target options"
    );
}

/// Test xctrace record help compatibility with Python idb
#[test]
fn test_xctrace_record_help_python_compatibility() {
    common::build_agent_mobile();

    // Python idb
    let python_output = Command::new("idb")
        .args(["xctrace", "record", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctrace", "record", "--help"])
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
    assert!(
        python_stdout.contains("all-processes") && rust_stdout.contains("all-processes"),
        "Both should mention all-processes"
    );
}
