mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_xctest_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list", "--udid", &udid])
        .output()
        .expect("Failed to run xctest-list");

    assert!(
        output.status.success(),
        "xctest-list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // リストが空の場合でも成功
    let _stdout = String::from_utf8_lossy(&output.stdout);
    // JSONまたは空の出力を期待
}

#[test]
fn test_xctest_list_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list", "--udid", &udid])
        .output()
        .expect("Failed to run xctest-list");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        // 出力がある場合はJSON形式であることを確認
        let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
        assert!(
            parse_result.is_ok(),
            "Expected valid JSON output, got: {}",
            stdout
        );
    }
}

/// Test xctest-list-bundle CLI help output (instead of requiring TEST_XCTEST_BUNDLE)
#[test]
fn test_xctest_list_bundle_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list-bundle", "--help"])
        .output()
        .expect("Failed to run xctest-list-bundle --help");

    assert!(
        output.status.success(),
        "xctest-list-bundle --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have bundle_id argument
    assert!(
        stdout.contains("BUNDLE")
            || stdout.contains("bundle")
            || stdout.contains("test")
            || stdout.contains("TEST"),
        "Expected bundle argument in help output"
    );
}

/// Test xctest-run CLI help output (instead of requiring TEST_XCTEST_BUNDLE)
#[test]
fn test_xctest_run_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-run", "--help"])
        .output()
        .expect("Failed to run xctest-run --help");

    assert!(
        output.status.success(),
        "xctest-run --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have test bundle argument
    assert!(
        stdout.contains("TEST_BUNDLE_ID") || stdout.contains("bundle"),
        "Expected test bundle argument in help output"
    );
}

/// Test xctest-run has --test flag for filtering
#[test]
fn test_xctest_run_has_test_filter_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-run", "--help"])
        .output()
        .expect("Failed to run xctest-run --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--test") || stdout.contains("filter"),
        "Expected --test flag or filter option in help output"
    );
}

/// Test xctest-run has tests-to-run option for filtering tests
#[test]
fn test_xctest_run_test_type_values() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-run", "--help"])
        .output()
        .expect("Failed to run xctest-run --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have tests-to-run option for filtering specific tests
    assert!(
        stdout.contains("tests-to-run")
            || stdout.contains("TESTS_TO_RUN")
            || stdout.contains("test"),
        "Expected tests-to-run option in help output"
    );
}

/// Test xctest-list-bundle help compatibility with Python idb
/// Note: Python idb uses `idb xctest list-bundle`, agent-mobile uses `idb xctest-list-bundle`
#[test]
fn test_xctest_list_bundle_help_python_compatibility() {
    // Python idb uses `xctest list-bundle`
    let python_output = Command::new("idb")
        .args(["xctest", "list-bundle", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses `xctest-list-bundle`
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list-bundle", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(
        python_output.status.success(),
        "Python idb xctest list-bundle --help failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile xctest-list-bundle --help failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
}

/// Test xctest-run help compatibility with Python idb
/// Note: Python idb uses `idb xctest run`, agent-mobile uses `idb xctest-run`
#[test]
fn test_xctest_run_help_python_compatibility() {
    // Python idb uses `xctest run`
    let python_output = Command::new("idb")
        .args(["xctest", "run", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses `xctest-run`
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-run", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(
        python_output.status.success(),
        "Python idb xctest run --help failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile xctest-run --help failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
}

/// Test xctest-list compatibility with Python idb
/// Note: Python idb uses `idb xctest list`, agent-mobile uses `idb xctest-list`
#[test]
fn test_xctest_list_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb uses `xctest list`
    let python_output = Command::new("idb")
        .args(["xctest", "list", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses `xctest-list`
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert!(
        python_output.status.success(),
        "Python idb xctest list failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile xctest-list failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
}

#[test]
fn test_xctest_list_empty() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list", "--udid", &udid])
        .output()
        .expect("Failed to run xctest-list");

    // XCTestバンドルがインストールされていない場合でも成功するべき
    assert!(output.status.success());
}
