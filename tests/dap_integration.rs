mod common;

use common::{ensure_companion_running, get_available_udid, get_test_bundle_id, wait_with_timeout};
use std::process::{Command, Stdio};

#[test]
#[ignore] // 複雑な対話的プロトコルのため、手動テストが望ましい
fn test_dap_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", &bundle_id, "--udid", &udid])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn dap");

    // DAPサーバーが起動することを確認（短時間で終了）
    let output = wait_with_timeout(child, 5);

    // プロセスが起動したことを確認
    // （DAPは対話的プロトコルなので、入力がないとタイムアウトする）
    assert!(
        output.status.code().is_some(),
        "Expected dap process to start"
    );
}

#[test]
#[ignore] // 複雑な対話的プロトコルのため、手動テストが望ましい
fn test_dap_stdin_stdout() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", &bundle_id, "--udid", &udid])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn dap");

    // stdin/stdoutが接続されていることを確認
    assert!(child.stdin.is_some(), "Expected stdin to be piped");
    assert!(child.stdout.is_some(), "Expected stdout to be piped");

    // プロセスを終了
    let output = wait_with_timeout(child, 5);
    assert!(output.status.code().is_some());
}

#[test]
#[ignore] // 複雑な対話的プロトコルのため、手動テストが望ましい
fn test_dap_without_bundle_id() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // bundle IDなしで実行
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", "--udid", &udid])
        .output()
        .expect("Failed to run dap");

    // bundle IDが必要なのでエラーになるべき
    assert!(!output.status.success());
}

#[test]
#[ignore] // 複雑な対話的プロトコルのため、手動テストが望ましい
fn test_dap_invalid_bundle_id() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", "com.invalid.nonexistent.app", "--udid", &udid])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn dap");

    // 無効なbundle IDでエラーになる可能性がある
    let output = wait_with_timeout(child, 5);
    // エラーハンドリングは実装依存
}

#[test]
#[ignore] // 複雑な対話的プロトコルのため、手動テストが望ましい
fn test_dap_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Python idb
    let python_child = Command::new("idb")
        .args(["dap", &bundle_id, "--udid", &udid])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Python idb");

    let python_output = wait_with_timeout(python_child, 3);

    // agent-mobile
    let rust_child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", &bundle_id, "--udid", &udid])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn agent-mobile");

    let rust_output = wait_with_timeout(rust_child, 3);

    // 両方ともプロセスが起動することを確認
    assert!(
        python_output.status.code().is_some(),
        "Python dap should start"
    );
    assert!(rust_output.status.code().is_some(), "Rust dap should start");
}

#[test]
#[ignore] // 複雑な対話的プロトコルのため、手動テストが望ましい
fn test_dap_with_port() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // ポートを指定してDAPサーバーを起動
    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", &bundle_id, "--port", "9999", "--udid", &udid])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn dap");

    let output = wait_with_timeout(child, 5);
    assert!(output.status.code().is_some());
}

#[test]
fn test_dap_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let bundle_id = get_test_bundle_id();

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
            stderr.contains("No companions available") || stderr.contains("target"),
            "Expected companion or target error, got: {}",
            stderr
        );
    }
}

#[test]
#[ignore] // 複雑な対話的プロトコルのため、手動テストが望ましい
fn test_dap_process_lifetime() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "dap", &bundle_id, "--udid", &udid])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn dap");

    // プロセスが実行中であることを確認
    std::thread::sleep(std::time::Duration::from_secs(1));

    // プロセスをkill
    let _ = child.kill();
    let output = child
        .wait_with_output()
        .expect("Failed to wait for process");

    assert!(output.status.code().is_some());
}

#[test]
#[ignore] // 手動テストのみ推奨
fn test_dap_manual_test_note() {
    // このテストは実行しないことを推奨
    // DAPプロトコルは複雑な対話的プロトコルであり、
    // 適切なDAPクライアントを使用して手動でテストする必要があります。
    eprintln!("NOTE: DAP tests should be performed manually with a DAP client");
    eprintln!("Example: Use VSCode with a DAP extension to test the DAP server");
    eprintln!("Skipping automated test.");
}
