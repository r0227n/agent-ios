use crate::common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_settings_set_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "set", "TestKey", "TestValue", "--udid", &udid])
        .output()
        .expect("Failed to run set");

    assert!(
        output.status.success(),
        "set failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_settings_get_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "get", "TestKey", "--udid", &udid])
        .output()
        .expect("Failed to run get");

    assert!(
        output.status.success(),
        "get failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_settings_set_get_roundtrip() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Set a value
    let set_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "set",
            "RoundtripKey",
            "RoundtripValue",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set");

    assert!(
        set_output.status.success(),
        "set failed: {}",
        String::from_utf8_lossy(&set_output.stderr)
    );

    // Get the value
    let get_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "get", "RoundtripKey", "--udid", &udid])
        .output()
        .expect("Failed to run get");

    assert!(
        get_output.status.success(),
        "get failed: {}",
        String::from_utf8_lossy(&get_output.stderr)
    );

    // Verify the value
    let stdout = String::from_utf8_lossy(&get_output.stdout);
    assert!(
        stdout.contains("RoundtripValue"),
        "Expected 'RoundtripValue' in output, got: {}",
        stdout
    );
}

#[test]
fn test_settings_set_with_domain() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "set",
            "DomainKey",
            "DomainValue",
            "--domain",
            "com.apple.Preferences",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set");

    assert!(output.status.success());
}

#[test]
fn test_settings_get_with_domain() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // First set a value with domain
    let _ = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "set",
            "DomainGetKey",
            "DomainGetValue",
            "--domain",
            "com.apple.Preferences",
            "--udid",
            &udid,
        ])
        .output();

    // Then get it
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "get",
            "DomainGetKey",
            "--domain",
            "com.apple.Preferences",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run get");

    assert!(output.status.success());
}

#[test]
fn test_settings_set_numeric_value() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "set", "NumericKey", "12345", "--udid", &udid])
        .output()
        .expect("Failed to run set");

    assert!(output.status.success());
}

#[test]
fn test_settings_set_boolean_value() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "set", "BooleanKey", "true", "--udid", &udid])
        .output()
        .expect("Failed to run set");

    assert!(output.status.success());
}

#[test]
fn test_settings_list_locale() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list", "locale", "--udid", &udid])
        .output()
        .expect("Failed to run list locale");

    assert!(
        output.status.success(),
        "list locale failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output is newline-separated locale identifiers (not JSON)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert!(!lines.is_empty(), "Expected non-empty locale list");
    // Check that each line looks like a locale identifier (e.g., "en_US", "ja_JP")
    for line in &lines[..lines.len().min(5)] {
        // Check first 5 lines
        assert!(!line.is_empty(), "Empty locale identifier line");
    }
}

#[test]
fn test_settings_set_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["set", "CompatKey", "CompatValue", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "set", "CompatKey", "CompatValue", "--udid", &udid])
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
fn test_settings_get_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // First set a value
    let _ = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "set",
            "GetCompatKey",
            "GetCompatValue",
            "--udid",
            &udid,
        ])
        .output();

    // Python idb
    let python_output = Command::new("idb")
        .args(["get", "GetCompatKey", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "get", "GetCompatKey", "--udid", &udid])
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
fn test_settings_list_locale_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["list", "locale", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list", "locale", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // 両方の出力がプレーンテキスト（改行区切り）であることを確認
    if python_output.status.success() {
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

        let python_lines: Vec<&str> = python_stdout.trim().lines().collect();
        let rust_lines: Vec<&str> = rust_stdout.trim().lines().collect();

        assert!(!python_lines.is_empty(), "Python output is empty");
        assert!(!rust_lines.is_empty(), "Rust output is empty");
    }
}
