mod common;

use common::{ensure_companion_running, get_available_udid, get_test_bundle_id};
use std::process::Command;

#[test]
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_start() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "start", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run debugserver start");

    assert!(
        output.status.success(),
        "debugserver start failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // クリーンアップ: デバッグサーバーを停止
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
        .output();
}

#[test]
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_status() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver status");

    assert!(
        output.status.success(),
        "debugserver status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // ステータス情報が出力される
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 出力形式は実装依存
}

#[test]
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_stop() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver stop");

    assert!(
        output.status.success(),
        "debugserver stop failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_start_stop_cycle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Start debug server
    let start_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "start", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run debugserver start");
    assert!(start_output.status.success());

    // Check status
    let status_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver status");
    assert!(status_output.status.success());

    // Stop debug server
    let stop_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver stop");
    assert!(stop_output.status.success());
}

#[test]
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_start_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Python idb
    let python_output = Command::new("idb")
        .args(["debugserver", "start", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // クリーンアップ
    let _ = Command::new("idb")
        .args(["debugserver", "stop", "--udid", &udid])
        .output();

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "start", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // クリーンアップ
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
        .output();

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}

#[test]
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_status_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["debugserver", "status", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status", "--udid", &udid])
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
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_stop_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["debugserver", "stop", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
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
#[ignore] // デバッグ可能なアプリが必要
fn test_debugserver_stop_not_running() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // デバッグサーバーが起動していない状態でstop
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "stop", "--udid", &udid])
        .output()
        .expect("Failed to run debugserver stop");

    // エラーにならない（または適切にハンドリングされる）
    // 実装によっては成功する場合もある
}

#[test]
fn test_debugserver_status_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "debugserver", "status"])
        .output()
        .expect("Failed to run debugserver status");

    // デフォルトターゲットがある場合は成功、ない場合はエラー
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available") || stderr.contains("target"),
            "Expected companion or target error, got: {}",
            stderr
        );
    }
}
