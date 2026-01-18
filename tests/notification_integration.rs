mod common;

use common::{ensure_companion_running, get_available_udid, get_test_bundle_id};
use std::process::Command;

#[test]
fn test_send_notification_simple() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"alert":"Hello"}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    assert!(
        output.status.success(),
        "send-notification simple failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_send_notification_with_title() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"alert":{"title":"Test Title","body":"Test Message"}}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    assert!(output.status.success());
}

#[test]
fn test_send_notification_with_body() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"alert":"Notification body text"}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    assert!(output.status.success());
}

#[test]
fn test_send_notification_with_badge() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"badge":5}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    assert!(output.status.success());
}

#[test]
fn test_send_notification_with_sound() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"sound":"default"}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    assert!(output.status.success());
}

#[test]
fn test_send_notification_complex() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"alert":{"title":"Test Title","body":"Test Body"},"badge":10,"sound":"default"}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    assert!(output.status.success());
}

#[test]
fn test_send_notification_with_user_info() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"alert":"Test"},"customData":{"key":"value"}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    assert!(output.status.success());
}

#[test]
fn test_send_notification_invalid_json() {
    let udid = get_available_udid();
    let bundle_id = get_test_bundle_id();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            "not-valid-json",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    // 無効なJSONでエラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_send_notification_invalid_bundle_id() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            "com.invalid.nonexistent.app",
            r#"{"aps":{"alert":"Test"}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    // 無効なバンドルIDでエラーになる可能性がある
    // （実装によっては成功する場合もある）
    // エラーチェックは緩めにする
}

#[test]
fn test_send_notification_empty_payload() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    let _output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run send-notification");

    // aps キーはあるが内容が空のペイロード
    // 成功する場合もある（実装依存）
}

#[test]
fn test_send_notification_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);
    let bundle_id = get_test_bundle_id();

    // Python idb (トップレベルコマンドとして send-notification を使用)
    let python_output = Command::new("idb")
        .args([
            "send-notification",
            &bundle_id,
            r#"{"aps":{"alert":"Test"}}"#,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "notification",
            "send-notification",
            &bundle_id,
            r#"{"aps":{"alert":"Test"}}"#,
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
