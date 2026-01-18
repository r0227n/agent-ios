mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_hid_tap() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "tap", "100", "200", "--udid", &udid])
        .output()
        .expect("Failed to run hid tap");

    assert!(
        output.status.success(),
        "hid tap failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // HID コマンドは成功時に何も出力しない
    assert!(
        output.stdout.is_empty(),
        "Expected no stdout output, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_hid_tap_with_duration() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "tap",
            "100",
            "200",
            "--duration",
            "0.5",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run hid tap with duration");

    assert!(output.status.success());
}

#[test]
fn test_hid_button_home() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "button", "HOME", "--udid", &udid])
        .output()
        .expect("Failed to run hid button");

    assert!(
        output.status.success(),
        "hid button failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.stdout.is_empty());
}

#[test]
fn test_hid_button_invalid() {
    let udid = get_available_udid();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "button", "INVALID_BUTTON", "--udid", &udid])
        .output()
        .expect("Failed to run hid button");

    assert!(!output.status.success());
}

#[test]
fn test_hid_key() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "key", "40", "--udid", &udid]) // Enter key
        .output()
        .expect("Failed to run hid key");

    assert!(output.status.success());
}

#[test]
fn test_hid_key_sequence() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "key-sequence", "4", "5", "6", "--udid", &udid]) // a, b, c
        .output()
        .expect("Failed to run hid key-sequence");

    assert!(output.status.success());
}

#[test]
fn test_hid_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "text", "Hello", "--udid", &udid])
        .output()
        .expect("Failed to run hid text");

    assert!(
        output.status.success(),
        "hid text failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.stdout.is_empty());
}

#[test]
fn test_hid_text_with_special_chars() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "text", "Hello123!@#", "--udid", &udid])
        .output()
        .expect("Failed to run hid text");

    assert!(output.status.success());
}

#[test]
fn test_hid_text_invalid_char() {
    let udid = get_available_udid();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "text", "あいうえお", "--udid", &udid]) // 日本語
        .output()
        .expect("Failed to run hid text");

    // サポートされていない文字でエラー
    assert!(!output.status.success());
}

#[test]
fn test_hid_swipe() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "swipe", "100", "100", "200", "200", "--udid", &udid])
        .output()
        .expect("Failed to run hid swipe");

    assert!(
        output.status.success(),
        "hid swipe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.stdout.is_empty());
}

#[test]
fn test_hid_swipe_with_duration_and_delta() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "swipe",
            "100",
            "100",
            "200",
            "200",
            "--duration",
            "0.5",
            "--delta",
            "10",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run hid swipe");

    assert!(output.status.success());
}

#[test]
fn test_hid_without_udid() {
    // デフォルトターゲットでタップ実行
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "tap", "100", "200"])
        .output()
        .expect("Failed to run hid tap");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available")
                || stderr.contains("Multiple companions available"),
            "Expected companion error, got: {}",
            stderr
        );
    }
}
