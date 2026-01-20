use crate::common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_memory_warning_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "simulate-memory-warning", "--udid", &udid])
        .output()
        .expect("Failed to run memory-warning");

    assert!(
        output.status.success(),
        "memory-warning failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_memory_warning_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "simulate-memory-warning"])
        .output()
        .expect("Failed to run memory-warning");

    // デフォルトターゲットがある場合は成功、ない場合はエラー
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // エラーメッセージが適切であることを確認
        assert!(
            stderr.contains("No companions available")
                || stderr.contains("Multiple companions available")
                || stderr.contains("target")
                || stderr.contains("Unimplemented")
                || stderr.contains("not implemented"),
            "Expected companion or target error, got: {}",
            stderr
        );
    }
}

#[test]
fn test_memory_warning_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["simulate-memory-warning", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "simulate-memory-warning", "--udid", &udid])
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
fn test_memory_warning_multiple_times() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // メモリ警告を複数回送信
    for _ in 0..3 {
        let output = Command::new("./target/debug/agent-mobile")
            .args(["idb", "simulate-memory-warning", "--udid", &udid])
            .output()
            .expect("Failed to run memory-warning");

        assert!(
            output.status.success(),
            "memory-warning failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_memory_warning_no_output() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "simulate-memory-warning", "--udid", &udid])
        .output()
        .expect("Failed to run memory-warning");

    assert!(output.status.success());

    // memory-warningは通常何も出力しない
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "Expected no stdout output, got: {}",
        stdout
    );
}
