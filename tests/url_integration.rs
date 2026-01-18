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

/// Note: tel:// URL scheme requires Phone app which is not available on iOS Simulator
/// This test verifies that the command runs correctly and handles the error appropriately
#[test]
fn test_url_open_tel() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "url", "open", "tel://1234567890", "--udid", &udid])
        .output()
        .expect("Failed to run url open");

    // tel:// scheme is not supported on iOS Simulator (no Phone app)
    // Accept either success (if somehow handled) or failure with appropriate error
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("kLSApplicationNotFoundErr")
                || stderr.contains("Failed to open URL")
                || stderr.contains("no application claims"),
            "Expected URL scheme not supported error, got: {}",
            stderr
        );
    }
}

/// Note: mailto: URL scheme requires Mail app which is not available on iOS Simulator
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

    // mailto: scheme is not supported on iOS Simulator (no Mail app)
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("kLSApplicationNotFoundErr")
                || stderr.contains("Failed to open URL")
                || stderr.contains("no application claims"),
            "Expected URL scheme not supported error, got: {}",
            stderr
        );
    }
}

/// Note: prefs: URL scheme behavior may vary on iOS Simulator
#[test]
fn test_url_open_settings() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 設定アプリを開く
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "url", "open", "prefs:root=General", "--udid", &udid])
        .output()
        .expect("Failed to run url open");

    // prefs: scheme may not be fully supported on iOS Simulator
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("kLSApplicationNotFoundErr")
                || stderr.contains("Failed to open URL")
                || stderr.contains("no application claims"),
            "Expected URL scheme not supported error, got: {}",
            stderr
        );
    }
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

/// Test custom URL scheme handling
/// Note: Custom URL schemes require a registered app which is not available by default
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

    // カスタムスキームのアプリがインストールされていない場合はエラー
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("kLSApplicationNotFoundErr")
                || stderr.contains("Failed to open URL")
                || stderr.contains("no application claims"),
            "Expected URL scheme not supported error, got: {}",
            stderr
        );
    }
}

/// Test URL open compatibility with Python idb
/// Note: Python idb uses `idb open <url>`, agent-mobile uses `idb url open <url>`
#[test]
fn test_url_open_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb uses `open` without `url` subcommand
    let python_output = Command::new("idb")
        .args(["open", "https://www.apple.com", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile uses `url open`
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
    assert!(
        python_output.status.success(),
        "Python idb open failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile url open failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );
}
