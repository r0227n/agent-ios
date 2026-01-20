use crate::common::{ensure_companion_running, get_available_udid};
use std::process::Command;

// ==================== UI Tap Tests ====================

#[test]
fn test_ui_tap() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "tap", "100", "200", "--udid", &udid])
        .output()
        .expect("Failed to run ui tap");

    assert!(
        output.status.success(),
        "ui tap failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        output.stdout.is_empty(),
        "Expected no stdout output, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_ui_tap_with_duration() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "ui",
            "tap",
            "100",
            "200",
            "--duration",
            "0.5",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run ui tap with duration");

    assert!(output.status.success());
}

// ==================== UI Button Tests ====================

#[test]
fn test_ui_button_home() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "button", "HOME", "--udid", &udid])
        .output()
        .expect("Failed to run ui button");

    assert!(
        output.status.success(),
        "ui button failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.stdout.is_empty());
}

#[test]
fn test_ui_button_invalid() {
    let udid = get_available_udid();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "button", "INVALID_BUTTON", "--udid", &udid])
        .output()
        .expect("Failed to run ui button");

    assert!(!output.status.success());
}

// ==================== UI Key Tests ====================

#[test]
fn test_ui_key() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "key", "40", "--udid", &udid]) // Enter key
        .output()
        .expect("Failed to run ui key");

    assert!(output.status.success());
}

#[test]
fn test_ui_key_with_duration() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "ui",
            "key",
            "40",
            "--duration",
            "0.5",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run ui key with duration");

    assert!(output.status.success());
}

// ==================== UI Key Sequence Tests ====================

#[test]
fn test_ui_key_sequence() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "key-sequence", "4", "5", "6", "--udid", &udid]) // a, b, c
        .output()
        .expect("Failed to run ui key-sequence");

    assert!(output.status.success());
}

// ==================== UI Text Tests ====================

#[test]
fn test_ui_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "text", "Hello", "--udid", &udid])
        .output()
        .expect("Failed to run ui text");

    assert!(
        output.status.success(),
        "ui text failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.stdout.is_empty());
}

#[test]
fn test_ui_text_with_special_chars() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "text", "Hello123!@#", "--udid", &udid])
        .output()
        .expect("Failed to run ui text");

    assert!(output.status.success());
}

#[test]
fn test_ui_text_invalid_char() {
    let udid = get_available_udid();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "text", "あいうえお", "--udid", &udid])
        .output()
        .expect("Failed to run ui text");

    assert!(!output.status.success());
}

// ==================== UI Swipe Tests ====================

#[test]
fn test_ui_swipe() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb", "ui", "swipe", "100", "100", "200", "200", "--udid", &udid,
        ])
        .output()
        .expect("Failed to run ui swipe");

    assert!(
        output.status.success(),
        "ui swipe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.stdout.is_empty());
}

#[test]
fn test_ui_swipe_with_duration_and_delta() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "ui",
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
        .expect("Failed to run ui swipe");

    assert!(output.status.success());
}

// ==================== UI Describe All Tests ====================

#[test]
fn test_ui_describe_all() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "describe-all", "--udid", &udid])
        .output()
        .expect("Failed to run ui describe-all");

    assert!(
        output.status.success(),
        "ui describe-all failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Expected accessibility info in stdout");
}

#[test]
fn test_ui_describe_all_nested() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "describe-all", "--nested", "--udid", &udid])
        .output()
        .expect("Failed to run ui describe-all --nested");

    assert!(
        output.status.success(),
        "ui describe-all --nested failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Expected accessibility info in stdout");
}

// ==================== UI Describe Point Tests ====================

#[test]
fn test_ui_describe_point() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "describe-point", "100", "200", "--udid", &udid])
        .output()
        .expect("Failed to run ui describe-point");

    assert!(
        output.status.success(),
        "ui describe-point failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Expected accessibility info in stdout");
}

#[test]
fn test_ui_describe_point_nested() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "ui",
            "describe-point",
            "100",
            "200",
            "--nested",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run ui describe-point --nested");

    assert!(
        output.status.success(),
        "ui describe-point --nested failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ==================== Python Compatibility Tests ====================

#[test]
fn test_ui_tap_compatibility_with_python() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb uses: idb ui tap <x> <y>
    let python_output = Command::new("idb")
        .args(["ui", "tap", "100", "200", "--udid", &udid])
        .output()
        .expect("Failed to run Python idb ui tap");

    // Rust agent-mobile uses: idb ui tap <x> <y>
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "tap", "100", "200", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile ui tap");

    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ:\n  Python idb: {:?}\n  agent-mobile: {:?}",
        python_output.status.success(),
        rust_output.status.success()
    );
}

#[test]
fn test_ui_describe_all_compatibility_with_python() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb uses: idb ui describe-all
    let python_output = Command::new("idb")
        .args(["ui", "describe-all", "--udid", &udid])
        .output()
        .expect("Failed to run Python idb ui describe-all");

    // Rust agent-mobile uses: idb ui describe-all
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "ui", "describe-all", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile ui describe-all");

    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // Both should return accessibility info
    if python_output.status.success() {
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
        assert_eq!(
            python_stdout.trim(),
            rust_stdout.trim(),
            "stdout differs:\n  Python idb:\n{}\n  agent-mobile:\n{}",
            python_stdout,
            rust_stdout
        );
    }
}
