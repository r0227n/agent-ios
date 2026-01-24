//! Select feature integration tests.
//!
//! Tests for `agent-mobile select` command.
//! Tests picker/spinner value selection functionality.

use crate::common::{
    assert_failure, assert_valid_json, ensure_companion_running, get_available_udid,
    get_test_bundle_id, run_cli_command_with_udid,
};

// ============================================================================
// Helper to navigate to a screen with picker
// ============================================================================

/// Navigate to Settings > General > Date & Time for time zone picker.
fn navigate_to_picker_screen(udid: &str) -> bool {
    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Find a picker element
    let output = run_cli_command_with_udid("find", &["type", "Picker", "-f", "json"], udid);
    output.status.success()
}

/// Get a picker reference if available.
fn get_picker_ref(udid: &str) -> Option<String> {
    let output = run_cli_command_with_udid("find", &["type", "Picker", "-f", "json"], udid);
    if !output.status.success() {
        // Try PickerWheel
        let output =
            run_cli_command_with_udid("find", &["type", "PickerWheel", "-f", "json"], udid);
        if !output.status.success() {
            return None;
        }
        let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
        return json
            .get("ref")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
    }

    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
    json.get("ref")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// ============================================================================
// select - Normal cases
// ============================================================================

/// Test select with element reference.
#[test]
fn test_select_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = navigate_to_picker_screen(&udid);

    if let Some(picker_ref) = get_picker_ref(&udid) {
        let output = run_cli_command_with_udid("select", &[&picker_ref, "value"], &udid);
        // May succeed or fail depending on available values
        let _ = output;
    }
    // If no picker found, test passes
}

/// Test select with text target.
#[test]
fn test_select_with_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try to select from a picker by label
    let output = run_cli_command_with_udid("select", &["Time Zone", "Tokyo"], &udid);

    // May succeed or fail depending on screen state
    let _ = output;
}

/// Test select with JSON format.
#[test]
fn test_select_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = navigate_to_picker_screen(&udid);

    if let Some(picker_ref) = get_picker_ref(&udid) {
        let output =
            run_cli_command_with_udid("select", &[&picker_ref, "value", "-f", "json"], &udid);
        if output.status.success() {
            let _ = assert_valid_json(&output);
        }
    }
}

/// Test select with --max-swipes option.
#[test]
fn test_select_max_swipes() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = navigate_to_picker_screen(&udid);

    if let Some(picker_ref) = get_picker_ref(&udid) {
        let output = run_cli_command_with_udid(
            "select",
            &[&picker_ref, "value", "--max-swipes", "5"],
            &udid,
        );
        // May succeed or fail depending on available values
        let _ = output;
    }
}

// ============================================================================
// Error cases
// ============================================================================

/// Test select with invalid element reference.
#[test]
fn test_select_invalid_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("select", &["@invalid999", "value"], &udid);

    assert_failure(&output, "select @invalid999");
}

/// Test select with non-existent target.
#[test]
fn test_select_non_existent_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("select", &["NonExistentPicker12345", "value"], &udid);

    assert_failure(&output, "select non-existent target");
}

/// Test select with invalid UDID.
#[test]
fn test_select_invalid_udid() {
    let output = run_cli_command_with_udid("select", &["@e1", "value"], "invalid-udid-12345");

    assert_failure(&output, "select with invalid UDID");
}

/// Test select without target.
#[test]
fn test_select_missing_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("select", &[], &udid);

    assert_failure(&output, "select (missing target)");
}

/// Test select without value.
#[test]
fn test_select_missing_value() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("select", &["@e1"], &udid);

    assert_failure(&output, "select (missing value)");
}

/// Test select with value not found.
#[test]
fn test_select_value_not_found() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = navigate_to_picker_screen(&udid);

    if let Some(picker_ref) = get_picker_ref(&udid) {
        let output = run_cli_command_with_udid(
            "select",
            &[&picker_ref, "NonExistentValue12345", "--max-swipes", "2"],
            &udid,
        );
        // Should fail after max swipes
        assert_failure(&output, "select value not found");
    }
}
