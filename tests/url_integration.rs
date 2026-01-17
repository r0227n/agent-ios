mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_url_open_https() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "url",
            "open",
            "https://www.apple.com",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run url open");

    assert!(
        output.status.success(),
        "url open https failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_url_open_http() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "url",
            "open",
            "http://www.example.com",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run url open");

    assert!(output.status.success());
}

#[test]
fn test_url_open_maps() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "url", "open", "maps://?q=Tokyo", "--udid", &udid])
        .output()
        .expect("Failed to run url open");

    assert!(output.status.success());
}

#[test]
fn test_url_open_tel() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "url", "open", "tel://1234567890", "--udid", &udid])
        .output()
        .expect("Failed to run url open");

    assert!(output.status.success());
}

#[test]
fn test_url_open_mailto() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "url",
            "open",
            "mailto:test@example.com",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run url open");

    assert!(output.status.success());
}

#[test]
fn test_url_open_settings() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 設定アプリを開く
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "url", "open", "prefs:root=General", "--udid", &udid])
        .output()
        .expect("Failed to run url open");

    assert!(output.status.success());
}

#[test]
fn test_url_open_with_query_params() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "url",
            "open",
            "https://www.example.com?foo=bar&baz=qux",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run url open");

    assert!(output.status.success());
}

#[test]
fn test_url_open_with_fragment() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "url",
            "open",
            "https://www.example.com#section",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run url open");

    assert!(output.status.success());
}

#[test]
fn test_url_open_custom_scheme() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // カスタムURLスキーム
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "url",
            "open",
            "myapp://open?param=value",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run url open");

    // カスタムスキームのアプリがインストールされていない場合でもエラーにならない
    // （iOSが処理する）
    assert!(output.status.success());
}

#[test]
fn test_url_open_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["url", "open", "https://www.apple.com", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "url",
            "open",
            "https://www.apple.com",
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
