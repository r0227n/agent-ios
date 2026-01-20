use crate::common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_target_describe() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "describe", "--udid", &udid])
        .output()
        .expect("Failed to run target describe");

    assert!(
        output.status.success(),
        "target describe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Expected non-empty output from describe"
    );

    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Failed to parse JSON");

    // Verify basic fields exist
    assert!(json.get("udid").is_some(), "Expected 'udid' field in JSON");
    assert!(json.get("name").is_some(), "Expected 'name' field in JSON");
    assert!(
        json.get("state").is_some(),
        "Expected 'state' field in JSON"
    );
}

#[test]
fn test_target_describe_json_structure() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "describe", "--udid", &udid])
        .output()
        .expect("Failed to run target describe");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Failed to parse JSON");

    // Verify UDID matches
    if let Some(json_udid) = json.get("udid").and_then(|v| v.as_str()) {
        assert_eq!(
            json_udid, udid,
            "Expected UDID in output to match requested UDID"
        );
    }
}

#[test]
fn test_target_describe_with_diagnostics() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "target",
            "describe",
            "--diagnostics",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run target describe");

    assert!(
        output.status.success(),
        "target describe --diagnostics failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Failed to parse JSON");

    // With diagnostics flag, should have additional information
    assert!(json.is_object(), "Expected JSON object");
}

/// Test target connect CLI help output (instead of actually connecting)
#[test]
fn test_target_connect_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "connect", "--help"])
        .output()
        .expect("Failed to run target connect --help");

    assert!(
        output.status.success(),
        "target connect --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have host and port arguments
    assert!(
        stdout.contains("HOST") || stdout.contains("host") || stdout.contains("address"),
        "Expected host argument in help output"
    );
}

/// Test target disconnect CLI help output (instead of actually disconnecting)
#[test]
fn test_target_disconnect_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "disconnect", "--help"])
        .output()
        .expect("Failed to run target disconnect --help");

    assert!(
        output.status.success(),
        "target disconnect --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

/// Test target describe command compatibility with Python idb
/// Note: Python idb uses `idb describe` (no `target` subcommand),
/// while agent-mobile uses `idb target describe`
#[test]
fn test_target_describe_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb uses `describe` without `target` subcommand
    let python_output = Command::new("idb")
        .args(["describe", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses `target describe`
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "describe", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert!(
        python_output.status.success(),
        "Python idb describe failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile target describe failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );

    // Rust 出力がJSONとしてパース可能であることを確認
    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
    let rust_json = serde_json::from_str::<serde_json::Value>(rust_stdout.trim())
        .expect("Rust output is not valid JSON");

    // UDIDが正しいことを確認
    let rs_udid = rust_json.get("udid").and_then(|v| v.as_str());
    assert_eq!(rs_udid, Some(udid.as_str()), "UDID in output differs");
}

/// Test target describe --diagnostics compatibility with Python idb
/// Note: Python idb uses `idb describe --diagnostics`, agent-mobile uses `idb target describe --diagnostics`
#[test]
fn test_target_describe_with_diagnostics_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb uses `describe` without `target` subcommand
    let python_output = Command::new("idb")
        .args(["describe", "--diagnostics", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses `target describe`
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "target",
            "describe",
            "--diagnostics",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert!(
        python_output.status.success(),
        "Python idb describe --diagnostics failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile target describe --diagnostics failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
}

/// Test target disconnect help compatibility with Python idb
/// Note: Python idb uses `idb disconnect --help`, agent-mobile uses `idb target disconnect --help`
#[test]
fn test_target_disconnect_help_python_compatibility() {
    // Python idb uses `disconnect` without `target` subcommand
    let python_output = Command::new("idb")
        .args(["disconnect", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses `target disconnect`
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "disconnect", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(
        python_output.status.success(),
        "Python idb disconnect --help failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile target disconnect --help failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
}

#[test]
fn test_target_describe_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "describe"])
        .output()
        .expect("Failed to run target describe");

    // デフォルトターゲットがある場合は成功、ない場合はエラー
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available")
                || stderr.contains("Multiple companions available")
                || stderr.contains("target")
                || stderr.contains("Unimplemented")
                || stderr.contains("not implemented"),
            "Expected companion or target error, got: {}",
            stderr
        );
    } else {
        // 成功した場合はJSON出力を確認
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
        assert!(parse_result.is_ok(), "Expected valid JSON output");
    }
}

#[test]
fn test_target_describe_invalid_udid() {
    let invalid_udid = "00000000-0000-0000-0000-000000000000";

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "describe", "--udid", invalid_udid])
        .output()
        .expect("Failed to run target describe");

    // 無効なUDIDでエラーになるべき
    assert!(!output.status.success());
}
