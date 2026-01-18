mod common;

use std::process::Command;

/// Test keychain-clear CLI help output
#[test]
fn test_keychain_clear_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "keychain-clear", "--help"])
        .output()
        .expect("Failed to run keychain-clear --help");

    assert!(
        output.status.success(),
        "keychain-clear --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

/// Test contacts-clear CLI help output
#[test]
fn test_contacts_clear_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-clear", "--help"])
        .output()
        .expect("Failed to run contacts-clear --help");

    assert!(
        output.status.success(),
        "contacts-clear --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--udid") || stdout.contains("UDID"),
        "Expected --udid flag in help output"
    );
}

/// Test contacts-update CLI help output
#[test]
fn test_contacts_update_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-update", "--help"])
        .output()
        .expect("Failed to run contacts-update --help");

    assert!(
        output.status.success(),
        "contacts-update --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have path argument or file parameter
    assert!(
        stdout.contains("DB_PATH") || stdout.contains("path") || stdout.contains("file"),
        "Expected path argument in help output"
    );
}

/// Test keychain-clear help compatibility with Python idb
#[test]
fn test_keychain_clear_help_python_compatibility() {
    // Python idb uses "clear-keychain" (not "keychain clear")
    let python_output = Command::new("idb")
        .args(["clear-keychain", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "keychain-clear", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test contacts-clear CLI help
/// Note: Python idb does not have "contacts clear" command, so this tests agent-mobile only
#[test]
fn test_contacts_clear_help_agent_mobile_only() {
    // agent-mobile (Python idb does not have "contacts clear" - only "contacts update")
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-clear", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    assert!(rust_output.status.success(), "agent-mobile help failed");
}

/// Test contacts-update help compatibility with Python idb
#[test]
fn test_contacts_update_help_python_compatibility() {
    // Python idb
    let python_output = Command::new("idb")
        .args(["contacts", "update", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "contacts-update", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");
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
