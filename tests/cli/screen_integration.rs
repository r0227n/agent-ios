//! Screen feature integration tests.
//!
//! Tests for `agent-mobile screen` commands.

use crate::common::{
    assert_stdout_contains, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, run_cli_command_with_udid,
};

/// Test screen tree command.
#[test]
fn test_screen_tree() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["tree"], &udid);

    assert_success(&output, "screen tree");
    // Tree should output accessibility tree (JSON format)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('{') || stdout.contains('['),
        "Expected JSON output, got: {}",
        stdout
    );
}

/// Test screen elements command.
#[test]
fn test_screen_elements() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["elements"], &udid);

    assert_success(&output, "screen elements");
    assert_stdout_contains(&output, "Screen Elements");
}

/// Test screen elements with JSON output.
#[test]
fn test_screen_elements_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["elements", "-o", "json"], &udid);

    assert_success(&output, "screen elements -o json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test screen hints command.
#[test]
fn test_screen_hints() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["hints"], &udid);

    assert_success(&output, "screen hints");
    assert_stdout_contains(&output, "Interactive Elements");
}
