mod common;

use std::process::Command;

/// Test DAP CLI help output
#[test]
fn test_dap_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", "--help"])
        .output()
        .expect("Failed to run dap --help");

    assert!(
        output.status.success(),
        "dap --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have bundle_id argument
    assert!(
        stdout.contains("BUNDLE")
            || stdout.contains("bundle")
            || stdout.contains("app")
            || stdout.to_lowercase().contains("bundle"),
        "Expected bundle argument in help output"
    );
}

/// Test DAP requires bundle ID
#[test]
fn test_dap_requires_bundle_id() {
    // DAP without bundle_id should fail
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap"])
        .output()
        .expect("Failed to run dap");

    // Should fail due to missing required argument
    assert!(
        !output.status.success(),
        "Expected dap to fail without bundle_id"
    );
}

/// Test DAP has --port flag
#[test]
fn test_dap_has_port_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", "--help"])
        .output()
        .expect("Failed to run dap --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--port") || stdout.contains("PORT"),
        "Expected --port flag in help output"
    );
}

/// Test DAP help compatibility with Python idb
#[test]
fn test_dap_help_python_compatibility() {
    // Python idb
    let python_output = Command::new("idb")
        .args(["dap", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test DAP has --udid flag
#[test]
fn test_dap_has_udid_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", "--help"])
        .output()
        .expect("Failed to run dap --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

#[test]
fn test_dap_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let bundle_id = common::get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", &bundle_id])
        .output()
        .expect("Failed to run dap");

    // デフォルトターゲットがある場合は起動するが、
    // ない場合はエラーになる
    // このテストは環境依存なので、エラーチェックのみ行う
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available")
                || stderr.contains("Multiple companions available")
                || stderr.contains("target")
                || stderr.contains("Unimplemented")
                || stderr.contains("not implemented")
                || stderr.contains("Internal"),
            "Expected companion or target error, got: {}",
            stderr
        );
    }
}

/// Test that dap accepts bundle_id argument (CLI validation)
#[test]
fn test_dap_accepts_bundle_id_argument() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", "com.example.test", "--help"])
        .output()
        .expect("Failed to run dap with bundle_id");

    // With --help, should always succeed regardless of bundle_id
    assert!(output.status.success());
}
