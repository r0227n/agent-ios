//! Navigator feature integration tests.
//!
//! Tests for `agent-mobile navigator` commands.

use crate::common::{
    assert_stdout_contains, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, get_test_bundle_id, run_cli_command_with_udid,
};

/// Test navigator list command.
#[test]
fn test_navigator_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("navigator", &["list"], &udid);

    assert_success(&output, "navigator list");
    assert_stdout_contains(&output, "Tappable elements");
}

/// Test navigator list with JSON output.
#[test]
fn test_navigator_list_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("navigator", &["list", "-o", "json"], &udid);

    assert_success(&output, "navigator list -o json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test navigator find command.
#[test]
fn test_navigator_find() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try to find a common element like "General" or "Settings"
    let output = run_cli_command_with_udid("navigator", &["find", "General"], &udid);

    // This might fail if "General" is not visible, but the command should complete
    // We check for either success or a known failure message
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success() || stderr.contains("No elements found"),
        "Expected success or 'No elements found', got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test navigator find-type command.
#[test]
fn test_navigator_find_type() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("navigator", &["find-type", "Button"], &udid);

    // Should find at least some buttons
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success() || stderr.contains("No elements found"),
        "Expected success or 'No elements found', got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

/// Test navigator find with --tap action.
#[test]
fn test_navigator_find_and_tap() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try to find and tap "General"
    let output = run_cli_command_with_udid("navigator", &["find", "General", "--tap"], &udid);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either it taps successfully or element not found
    assert!(
        output.status.success() || stderr.contains("No elements found"),
        "Expected success or 'No elements found', got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // If successful, check for tap confirmation
    if output.status.success() {
        assert_stdout_contains(&output, "Tapped element");
    }
}
