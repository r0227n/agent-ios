mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
#[ignore] // 破壊的な操作のため、デフォルトでは実行しない
fn test_keychain_clear() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "keychain-clear", "--udid", &udid])
        .output()
        .expect("Failed to run keychain-clear");

    assert!(
        output.status.success(),
        "keychain-clear failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // 破壊的な操作のため、デフォルトでは実行しない
fn test_contacts_clear() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-clear", "--udid", &udid])
        .output()
        .expect("Failed to run contacts-clear");

    assert!(
        output.status.success(),
        "contacts-clear failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // TEST_CONTACTS_DB環境変数が必要
fn test_contacts_update() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // TEST_CONTACTS_DB環境変数からDBファイルパスを取得
    let db_path = std::env::var("TEST_CONTACTS_DB").expect(
        "TEST_CONTACTS_DB environment variable must be set with path to contacts database file",
    );

    // ファイルが存在することを確認
    assert!(
        std::path::Path::new(&db_path).exists(),
        "Contacts database file does not exist: {}",
        db_path
    );

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-update", &db_path, "--udid", &udid])
        .output()
        .expect("Failed to run contacts-update");

    assert!(
        output.status.success(),
        "contacts-update failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // 破壊的な操作のため、デフォルトでは実行しない
fn test_keychain_clear_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["keychain-clear", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "keychain-clear", "--udid", &udid])
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
#[ignore] // 破壊的な操作のため、デフォルトでは実行しない
fn test_contacts_clear_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["contacts-clear", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-clear", "--udid", &udid])
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
#[ignore] // TEST_CONTACTS_DB環境変数が必要
fn test_contacts_update_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let db_path = std::env::var("TEST_CONTACTS_DB")
        .expect("TEST_CONTACTS_DB environment variable must be set");

    assert!(std::path::Path::new(&db_path).exists());

    // Python idb
    let python_output = Command::new("idb")
        .args(["contacts-update", &db_path, "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-update", &db_path, "--udid", &udid])
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
fn test_keychain_clear_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "keychain-clear"])
        .output()
        .expect("Failed to run keychain-clear");

    // デフォルトターゲットがある場合は成功、ない場合はエラー
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // エラーメッセージが適切であることを確認
        assert!(
            stderr.contains("No companions available") || stderr.contains("target"),
            "Expected companion or target error, got: {}",
            stderr
        );
    }
}

#[test]
fn test_contacts_clear_without_udid() {
    // UDIDなしで実行（デフォルトターゲット使用）
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-clear"])
        .output()
        .expect("Failed to run contacts-clear");

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
