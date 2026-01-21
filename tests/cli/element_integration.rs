//! Element feature integration tests.
//!
//! Tests for `agent-mobile element` commands.

use crate::common::{
    assert_stdout_contains, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, get_test_bundle_id, run_cli_command_with_udid,
};

/// Test element list command.
#[test]
fn test_element_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("element", &["list"], &udid);

    assert_success(&output, "element list");
    assert_stdout_contains(&output, "Tappable elements");
}

/// Test element list with JSON output.
#[test]
fn test_element_list_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("element", &["list", "-o", "json"], &udid);

    assert_success(&output, "element list -o json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test element find command.
#[test]
fn test_element_find() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try to find a common element like "General" or "Settings"
    let output = run_cli_command_with_udid("element", &["find", "General"], &udid);

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

/// Test element find-type command.
#[test]
fn test_element_find_type() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("element", &["find-type", "Button"], &udid);

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

/// Test element find with --tap action.
#[test]
fn test_element_find_and_tap() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try to find and tap "General"
    let output = run_cli_command_with_udid("element", &["find", "General", "--tap"], &udid);

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

/// Test element list with no scrolling (max-scrolls=0).
#[test]
fn test_element_list_no_scroll() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("element", &["list", "--max-scrolls", "0"], &udid);

    assert_success(&output, "element list --max-scrolls 0");
    assert_stdout_contains(&output, "Tappable elements");
}

/// Test element list with scrolling collection.
#[test]
fn test_element_list_with_scroll() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test with minimal scrolls to avoid long test times
    let output = run_cli_command_with_udid("element", &["list", "--max-scrolls", "1"], &udid);

    assert_success(&output, "element list --max-scrolls 1");
    assert_stdout_contains(&output, "Tappable elements");
}

/// Test element list with scrolling and JSON output.
#[test]
fn test_element_list_scroll_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid(
        "element",
        &["list", "--max-scrolls", "1", "-o", "json"],
        &udid,
    );

    assert_success(&output, "element list --max-scrolls 1 -o json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test element command defaults to list subcommand.
#[test]
fn test_element_default_to_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Run 'element' without subcommand - should default to 'list'
    let output = run_cli_command_with_udid("element", &[], &udid);

    assert_success(&output, "element (default to list)");
    assert_stdout_contains(&output, "Tappable elements");
}

/// Test element command defaults to list with JSON output.
#[test]
fn test_element_default_to_list_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app to have elements to list
    let bundle_id = get_test_bundle_id();
    let _ = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Run 'element -o json' without subcommand - should default to 'list'
    let output = run_cli_command_with_udid("element", &["-o", "json"], &udid);

    assert_success(&output, "element -o json (default to list)");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}
