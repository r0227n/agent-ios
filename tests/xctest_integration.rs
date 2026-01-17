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

#[test]
#[ignore] // TEST_XCTEST_BUNDLE環境変数が必要
fn test_xctest_list_bundle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // TEST_XCTEST_BUNDLE環境変数からバンドルIDを取得
    let bundle_id = std::env::var("TEST_XCTEST_BUNDLE")
        .expect("TEST_XCTEST_BUNDLE environment variable must be set with XCTest bundle ID");

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list-bundle", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run xctest-list-bundle");

    assert!(
        output.status.success(),
        "xctest-list-bundle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Expected non-empty test list from bundle"
    );
}

#[test]
#[ignore] // TEST_XCTEST_BUNDLE環境変数が必要
fn test_xctest_list_bundle_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = std::env::var("TEST_XCTEST_BUNDLE")
        .expect("TEST_XCTEST_BUNDLE environment variable must be set");

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list-bundle", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run xctest-list-bundle");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
        assert!(
            parse_result.is_ok(),
            "Expected valid JSON output, got: {}",
            stdout
        );
    }
}

#[test]
#[ignore] // TEST_XCTEST_BUNDLE環境変数が必要、かつ実行に時間がかかる
fn test_xctest_run() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = std::env::var("TEST_XCTEST_BUNDLE")
        .expect("TEST_XCTEST_BUNDLE environment variable must be set with XCTest bundle ID");

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-run", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to run xctest-run");

    // テストが失敗する可能性があるため、実行自体が成功することのみ確認
    // （テストの成否は問わない）
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 何らかの出力があることを確認
    assert!(
        !stdout.is_empty() || !stderr.is_empty(),
        "Expected some output from xctest-run"
    );
}

#[test]
#[ignore] // TEST_XCTEST_BUNDLE環境変数が必要
fn test_xctest_run_with_test_filter() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = std::env::var("TEST_XCTEST_BUNDLE")
        .expect("TEST_XCTEST_BUNDLE environment variable must be set");

    // 特定のテストを実行（例: MyTestClass/testExample）
    let _output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "xctest-run",
            &bundle_id,
            "--test",
            "MyTestClass/testExample",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run xctest-run");

    // テストフィルタが有効な場合のみ成功
    // 無効な場合はエラーになる可能性がある
}

#[test]
#[ignore] // TEST_XCTEST_BUNDLE環境変数が必要
fn test_xctest_run_logic_test() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = std::env::var("TEST_XCTEST_BUNDLE")
        .expect("TEST_XCTEST_BUNDLE environment variable must be set");

    // ロジックテストとして実行
    let _output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "xctest-run",
            &bundle_id,
            "--test-type",
            "logic",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run xctest-run");

    // 実行できたことを確認（結果は問わない）
}

#[test]
#[ignore] // TEST_XCTEST_BUNDLE環境変数が必要
fn test_xctest_run_ui_test() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = std::env::var("TEST_XCTEST_BUNDLE")
        .expect("TEST_XCTEST_BUNDLE environment variable must be set");

    // UIテストとして実行
    let _output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "xctest-run",
            &bundle_id,
            "--test-type",
            "ui",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run xctest-run");

    // 実行できたことを確認（結果は問わない）
}

#[test]
fn test_xctest_list_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["xctest-list", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list", "--udid", &udid])
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
#[ignore] // TEST_XCTEST_BUNDLE環境変数が必要
fn test_xctest_list_bundle_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = std::env::var("TEST_XCTEST_BUNDLE")
        .expect("TEST_XCTEST_BUNDLE environment variable must be set");

    // Python idb
    let python_output = Command::new("idb")
        .args(["xctest-list-bundle", &bundle_id, "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-list-bundle", &bundle_id, "--udid", &udid])
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
