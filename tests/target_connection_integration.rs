mod common;

use common::{ensure_companion_running, get_available_udid};
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

#[test]
#[ignore] // companionへの接続が必要で、環境依存
fn test_target_connect() {
    // テスト用のホストとポートを設定
    let host = "localhost";
    let port = "10882"; // デフォルトのidb companion port

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "connect", host, port])
        .output()
        .expect("Failed to run target connect");

    // 接続が成功するかは環境依存
    // エラーが適切にハンドリングされることを確認
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            !stderr.is_empty(),
            "Expected error message on connection failure"
        );
    }
}

#[test]
#[ignore] // 接続状態の変更を伴うため
fn test_target_disconnect() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "disconnect", "--udid", &udid])
        .output()
        .expect("Failed to run target disconnect");

    assert!(
        output.status.success(),
        "target disconnect failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 再接続
    let _ = Command::new("idb")
        .args(["describe", "--udid", &udid])
        .output();
}

#[test]
fn test_target_describe_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["target", "describe", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "describe", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // 両方の出力がJSONとしてパース可能であることを確認
    if python_output.status.success() {
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

        let python_json = serde_json::from_str::<serde_json::Value>(python_stdout.trim());
        let rust_json = serde_json::from_str::<serde_json::Value>(rust_stdout.trim());

        assert!(python_json.is_ok(), "Python output is not valid JSON");
        assert!(rust_json.is_ok(), "Rust output is not valid JSON");

        // 両方のJSONでUDIDが一致することを確認
        if let (Ok(py_json), Ok(rs_json)) = (python_json, rust_json) {
            let py_udid = py_json.get("udid").and_then(|v| v.as_str());
            let rs_udid = rs_json.get("udid").and_then(|v| v.as_str());
            assert_eq!(py_udid, rs_udid, "UDIDs in output differ");
        }
    }
}

#[test]
fn test_target_describe_with_diagnostics_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["target", "describe", "--diagnostics", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
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
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}

#[test]
#[ignore] // 接続状態の変更を伴うため
fn test_target_disconnect_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["target", "disconnect", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // 再接続
    let _ = Command::new("idb")
        .args(["describe", "--udid", &udid])
        .output();

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "disconnect", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 再接続
    let _ = Command::new("idb")
        .args(["describe", "--udid", &udid])
        .output();

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
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
            stderr.contains("No companions available") || stderr.contains("target"),
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
