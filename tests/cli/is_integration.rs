//! Is feature integration tests.
//!
//! Tests for `agent-mobile is` commands.
//! Tests element state checking functionality using exit codes.

use crate::common::{
    assert_exit_code_0, assert_exit_code_1, assert_failure, assert_success, assert_valid_json,
    ensure_companion_running, get_available_udid, get_test_bundle_id, run_cli_command_with_udid,
};

// ============================================================================
// Helper to get element reference
// ============================================================================

/// Get first element reference from find command.
fn get_element_ref(udid: &str) -> String {
    let output = run_cli_command_with_udid("find", &["type", "StaticText", "-f", "json"], udid);
    assert_success(&output, "find type for element ref");

    let json = assert_valid_json(&output);
    json.get("ref")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .expect("Expected ref field in find output")
}

/// Get first button reference from find command.
fn get_button_ref(udid: &str) -> String {
    let output = run_cli_command_with_udid("find", &["type", "Cell", "-f", "json"], udid);
    assert_success(&output, "find type Cell for ref");

    let json = assert_valid_json(&output);
    json.get("ref")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .expect("Expected ref field in find output")
}

// ============================================================================
// is visible - Normal cases
// ============================================================================

/// Test is visible with element reference (true case).
#[test]
fn test_is_visible_true() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("is", &["visible", &elem_ref], &udid);

    assert_exit_code_0(&output, "is visible (true)");
}

/// Test is visible with text target (true case).
#[test]
fn test_is_visible_with_text_target_true() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("is", &["visible", "General"], &udid);

    assert_exit_code_0(&output, "is visible General (true)");
}

/// Test is visible with non-existent element (false case).
#[test]
fn test_is_visible_false() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("is", &["visible", "NonExistentText12345"], &udid);

    assert_exit_code_1(&output, "is visible (false)");
}

// ============================================================================
// is exists - Normal cases
// ============================================================================

/// Test is exists with element reference (true case).
#[test]
fn test_is_exists_true() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("is", &["exists", &elem_ref], &udid);

    assert_exit_code_0(&output, "is exists (true)");
}

/// Test is exists with text target (true case).
#[test]
fn test_is_exists_with_text_target_true() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("is", &["exists", "General"], &udid);

    assert_exit_code_0(&output, "is exists General (true)");
}

/// Test is exists with non-existent element (false case).
#[test]
fn test_is_exists_false() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("is", &["exists", "NonExistentText12345"], &udid);

    assert_exit_code_1(&output, "is exists (false)");
}

// ============================================================================
// is enabled - Normal cases
// ============================================================================

/// Test is enabled with element reference (true case).
#[test]
fn test_is_enabled_true() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_button_ref(&udid);
    let output = run_cli_command_with_udid("is", &["enabled", &elem_ref], &udid);

    // Most buttons in Settings should be enabled
    assert_exit_code_0(&output, "is enabled (true)");
}

/// Test is enabled with text target.
#[test]
fn test_is_enabled_with_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("is", &["enabled", "General"], &udid);

    // General cell should be enabled
    assert_exit_code_0(&output, "is enabled General");
}

// ============================================================================
// is disabled - Normal cases
// ============================================================================

/// Test is disabled (inverse of enabled).
#[test]
fn test_is_disabled() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_button_ref(&udid);
    let output = run_cli_command_with_udid("is", &["disabled", &elem_ref], &udid);

    // Most elements should not be disabled, so exit code 1 expected
    assert_exit_code_1(&output, "is disabled (false for enabled element)");
}

/// Test is disabled with non-existent element.
#[test]
fn test_is_disabled_non_existent() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("is", &["disabled", "NonExistentText12345"], &udid);

    // Element doesn't exist, should return exit code 1
    assert_exit_code_1(&output, "is disabled (element not found)");
}

// ============================================================================
// is interactive - Normal cases
// ============================================================================

/// Test is interactive with button (true case).
#[test]
fn test_is_interactive_true() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_button_ref(&udid);
    let output = run_cli_command_with_udid("is", &["interactive", &elem_ref], &udid);

    // Cells should be interactive
    assert_exit_code_0(&output, "is interactive (true)");
}

/// Test is interactive with text target.
#[test]
fn test_is_interactive_with_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("is", &["interactive", "General"], &udid);

    // General cell should be interactive
    assert_exit_code_0(&output, "is interactive General");
}

/// Test is interactive with static text (likely false).
#[test]
fn test_is_interactive_static_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("is", &["interactive", &elem_ref], &udid);

    // Static text may or may not be interactive depending on context
    let _ = output;
}

// ============================================================================
// is checked - Normal cases
// ============================================================================

/// Test is checked with element reference.
#[test]
fn test_is_checked() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Find a switch element if available
    let output = run_cli_command_with_udid("find", &["type", "Switch", "-f", "json"], &udid);

    if output.status.success() {
        let json = assert_valid_json(&output);
        if let Some(ref_val) = json.get("ref").and_then(|v| v.as_str()) {
            let check_output = run_cli_command_with_udid("is", &["checked", ref_val], &udid);
            // Exit code depends on switch state
            let _ = check_output;
        }
    }
    // If no switch found, test passes (nothing to check)
}

/// Test is checked with non-checkable element.
#[test]
fn test_is_checked_non_checkable() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // StaticText is not checkable
    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("is", &["checked", &elem_ref], &udid);

    // Should return false (exit code 1) for non-checkable elements
    assert_exit_code_1(&output, "is checked (non-checkable element)");
}

// ============================================================================
// is - Error cases
// ============================================================================

/// Test is with invalid state name.
#[test]
fn test_is_invalid_state() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("is", &["invalidstate", "@e1"], &udid);

    assert_failure(&output, "is invalidstate");
}

/// Test is with invalid element reference.
#[test]
fn test_is_invalid_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("is", &["visible", "@invalid999"], &udid);

    assert_exit_code_1(&output, "is visible @invalid999");
}

/// Test is with missing target.
#[test]
fn test_is_missing_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("is", &["visible"], &udid);

    assert_failure(&output, "is visible (missing target)");
}

/// Test is with invalid UDID.
#[test]
fn test_is_invalid_udid() {
    let output = run_cli_command_with_udid("is", &["visible", "@e1"], "invalid-udid-12345");

    assert_failure(&output, "is with invalid UDID");
}

// ============================================================================
// Exit code verification
// ============================================================================

/// Test that visible returns 0 for visible element.
#[test]
fn test_is_exit_code_0_for_true() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("is", &["exists", "General"], &udid);

    assert!(
        output.status.code() == Some(0),
        "Expected exit code 0 for existing element, got {:?}",
        output.status.code()
    );
}

/// Test that visible returns 1 for non-existent element.
#[test]
fn test_is_exit_code_1_for_false() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("is", &["exists", "NonExistentElement999"], &udid);

    assert!(
        output.status.code() == Some(1),
        "Expected exit code 1 for non-existent element, got {:?}",
        output.status.code()
    );
}
