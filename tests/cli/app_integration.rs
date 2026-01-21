//! App feature integration tests.
//!
//! Tests for `agent-mobile app` commands.

use crate::common::{
    assert_stdout_contains, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, get_test_bundle_id, run_cli_command_with_udid,
};

/// Test app --list command.
#[test]
fn test_app_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("app", &["--list"], &udid);

    assert_success(&output, "app --list");
    assert_stdout_contains(&output, "Installed Apps");
}

/// Test app --list with JSON output.
#[test]
fn test_app_list_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("app", &["--list", "-o", "json"], &udid);

    assert_success(&output, "app --list -o json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test app --launch command.
#[test]
fn test_app_launch() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();
    let output = run_cli_command_with_udid("app", &["--launch", &bundle_id], &udid);

    assert_success(&output, "app --launch");
    assert_stdout_contains(&output, "Launched");
}

/// Test app --terminate command.
#[test]
fn test_app_terminate() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();

    // First launch the app
    let launch_output = run_cli_command_with_udid("app", &["--launch", &bundle_id], &udid);
    assert_success(&launch_output, "app --launch (setup)");

    // Small delay to ensure app is running
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Then terminate
    let output = run_cli_command_with_udid("app", &["--terminate", &bundle_id], &udid);

    assert_success(&output, "app --terminate");
    assert_stdout_contains(&output, "Terminated");
}
