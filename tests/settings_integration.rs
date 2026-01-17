mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_settings_set_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "settings",
            "set",
            "TestKey",
            "TestValue",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run settings set");

    assert!(
        output.status.success(),
        "settings set failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_settings_get_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "settings", "get", "TestKey", "--udid", &udid])
        .output()
        .expect("Failed to run settings get");

    assert!(
        output.status.success(),
        "settings get failed: {}",
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
            "settings",
            "set",
            "RoundtripKey",
            "RoundtripValue",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run settings set");

    assert!(
        set_output.status.success(),
        "settings set failed: {}",
        String::from_utf8_lossy(&set_output.stderr)
    );

    // Get the value
    let get_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "settings", "get", "RoundtripKey", "--udid", &udid])
        .output()
        .expect("Failed to run settings get");

    assert!(
        get_output.status.success(),
        "settings get failed: {}",
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
            "settings",
            "set",
            "DomainKey",
            "DomainValue",
            "--domain",
            "com.apple.Preferences",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run settings set");

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
            "settings",
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
            "settings",
            "get",
            "DomainGetKey",
            "--domain",
            "com.apple.Preferences",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run settings get");

    assert!(output.status.success());
}

#[test]
fn test_settings_set_numeric_value() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "settings",
            "set",
            "NumericKey",
            "12345",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run settings set");

    assert!(output.status.success());
}

#[test]
fn test_settings_set_boolean_value() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "settings",
            "set",
            "BooleanKey",
            "true",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run settings set");

    assert!(output.status.success());
}

#[test]
fn test_settings_list_locale() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "settings", "list-locale", "--udid", &udid])
        .output()
        .expect("Failed to run settings list-locale");

    assert!(
        output.status.success(),
        "settings list-locale failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Expected non-empty output from list-locale"
    );

    // Try to parse as JSON
    let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
    assert!(
        parse_result.is_ok(),
        "Expected valid JSON output, got: {}",
        stdout
    );
}

#[test]
fn test_settings_list_locale_json_structure() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "settings", "list-locale", "--udid", &udid])
        .output()
        .expect("Failed to run settings list-locale");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Failed to parse JSON");

    // Verify JSON is an array or object
    assert!(
        json.is_array() || json.is_object(),
        "Expected array or object, got: {:?}",
        json
    );
}

#[test]
fn test_settings_set_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args([
            "settings",
            "set",
            "CompatKey",
            "CompatValue",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "settings",
            "set",
            "CompatKey",
            "CompatValue",
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

#[test]
fn test_settings_get_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // First set a value
    let _ = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "settings",
            "set",
            "GetCompatKey",
            "GetCompatValue",
            "--udid",
            &udid,
        ])
        .output();

    // Python idb
    let python_output = Command::new("idb")
        .args(["settings", "get", "GetCompatKey", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "settings", "get", "GetCompatKey", "--udid", &udid])
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
        .args(["settings", "list-locale", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "settings", "list-locale", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // 両方の出力がJSONとしてパース可能であることを確認
    if python_output.status.success() {
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

        let python_json = serde_json::from_str::<serde_json::Value>(python_stdout.trim());
        let rust_json = serde_json::from_str::<serde_json::Value>(rust_stdout.trim());

        assert!(python_json.is_ok(), "Python output is not valid JSON");
        assert!(rust_json.is_ok(), "Rust output is not valid JSON");
    }
}
