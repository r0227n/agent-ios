//! Screen feature integration tests.
//!
//! Tests for `agent-mobile screen` commands.

use crate::common::{
    assert_stdout_contains, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, run_cli_command_with_udid,
};

/// Test screen --dump command.
#[test]
fn test_screen_dump() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["--dump"], &udid);

    assert_success(&output, "screen --dump");
    // Dump should output accessibility tree (JSON format)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('{') || stdout.contains('['),
        "Expected JSON output, got: {}",
        stdout
    );
}

/// Test screen --summary command.
#[test]
fn test_screen_summary() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["--summary"], &udid);

    assert_success(&output, "screen --summary");
    assert_stdout_contains(&output, "Screen Elements");
}

/// Test screen --summary with JSON output.
#[test]
fn test_screen_summary_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["--summary", "-o", "json"], &udid);

    assert_success(&output, "screen --summary -o json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test screen --hints command.
#[test]
fn test_screen_hints() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screen", &["--hints"], &udid);

    assert_success(&output, "screen --hints");
    assert_stdout_contains(&output, "Interactive Elements");
}
