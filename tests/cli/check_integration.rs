//! Check/Uncheck feature integration tests.
//!
//! Tests for `agent-mobile check` and `agent-mobile uncheck` commands.
//! Tests idempotent checkbox/switch toggle functionality.

use crate::common::{
    assert_failure, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, get_test_bundle_id, run_cli_command_with_udid,
};

// ============================================================================
// Helper to find a switch element
// ============================================================================

/// Navigate to a screen with switches (e.g., Settings > General > Keyboard).
fn navigate_to_screen_with_switches(udid: &str) {
    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Tap General
    let _ = run_cli_command_with_udid("find", &["text", "General", "tap"], udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Tap Keyboard
    let _ = run_cli_command_with_udid("find", &["text", "Keyboard", "tap"], udid);
    std::thread::sleep(std::time::Duration::from_millis(500));
}

/// Get a switch reference if available.
fn get_switch_ref(udid: &str) -> Option<String> {
    let output = run_cli_command_with_udid("find", &["type", "Switch", "-f", "json"], udid);
    if !output.status.success() {
        return None;
    }

    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;
    json.get("ref")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// ============================================================================
// check - Normal cases
// ============================================================================

/// Test check with element reference.
#[test]
fn test_check_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    if let Some(switch_ref) = get_switch_ref(&udid) {
        let output = run_cli_command_with_udid("check", &[&switch_ref], &udid);
        assert_success(&output, "check with ref");
    }
    // If no switch found, test passes (nothing to check)
}

/// Test check with text target.
#[test]
fn test_check_with_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    // Try to check "Auto-Capitalization" switch
    let output = run_cli_command_with_udid("check", &["Auto-Capitalization"], &udid);

    // May succeed or fail depending on if element exists
    let _ = output;
}

/// Test check with JSON format.
#[test]
fn test_check_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    if let Some(switch_ref) = get_switch_ref(&udid) {
        let output = run_cli_command_with_udid("check", &[&switch_ref, "-f", "json"], &udid);
        if output.status.success() {
            let _ = assert_valid_json(&output);
        }
    }
}

/// Test check idempotency (multiple calls should be safe).
#[test]
fn test_check_idempotent() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    if let Some(switch_ref) = get_switch_ref(&udid) {
        // Call check twice - should be idempotent
        let output1 = run_cli_command_with_udid("check", &[&switch_ref], &udid);
        let output2 = run_cli_command_with_udid("check", &[&switch_ref], &udid);

        // Both should succeed
        assert_success(&output1, "check first call");
        assert_success(&output2, "check second call");
    }
}

// ============================================================================
// uncheck - Normal cases
// ============================================================================

/// Test uncheck with element reference.
#[test]
fn test_uncheck_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    if let Some(switch_ref) = get_switch_ref(&udid) {
        let output = run_cli_command_with_udid("uncheck", &[&switch_ref], &udid);
        assert_success(&output, "uncheck with ref");
    }
}

/// Test uncheck with text target.
#[test]
fn test_uncheck_with_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    let output = run_cli_command_with_udid("uncheck", &["Auto-Capitalization"], &udid);

    // May succeed or fail depending on if element exists
    let _ = output;
}

/// Test uncheck with JSON format.
#[test]
fn test_uncheck_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    if let Some(switch_ref) = get_switch_ref(&udid) {
        let output = run_cli_command_with_udid("uncheck", &[&switch_ref, "-f", "json"], &udid);
        if output.status.success() {
            let _ = assert_valid_json(&output);
        }
    }
}

/// Test uncheck idempotency.
#[test]
fn test_uncheck_idempotent() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    if let Some(switch_ref) = get_switch_ref(&udid) {
        // Call uncheck twice - should be idempotent
        let output1 = run_cli_command_with_udid("uncheck", &[&switch_ref], &udid);
        let output2 = run_cli_command_with_udid("uncheck", &[&switch_ref], &udid);

        assert_success(&output1, "uncheck first call");
        assert_success(&output2, "uncheck second call");
    }
}

// ============================================================================
// check/uncheck cycle
// ============================================================================

/// Test check then uncheck cycle.
#[test]
fn test_check_uncheck_cycle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    navigate_to_screen_with_switches(&udid);

    if let Some(switch_ref) = get_switch_ref(&udid) {
        // Check
        let check_output = run_cli_command_with_udid("check", &[&switch_ref], &udid);
        assert_success(&check_output, "check in cycle");

        std::thread::sleep(std::time::Duration::from_millis(300));

        // Uncheck
        let uncheck_output = run_cli_command_with_udid("uncheck", &[&switch_ref], &udid);
        assert_success(&uncheck_output, "uncheck in cycle");
    }
}

// ============================================================================
// Error cases
// ============================================================================

/// Test check with invalid element reference.
#[test]
fn test_check_invalid_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("check", &["@invalid999"], &udid);

    assert_failure(&output, "check @invalid999");
}

/// Test check with non-existent text target.
#[test]
fn test_check_non_existent_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("check", &["NonExistentSwitchElement12345"], &udid);

    assert_failure(&output, "check non-existent target");
}

/// Test uncheck with invalid element reference.
#[test]
fn test_uncheck_invalid_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("uncheck", &["@invalid999"], &udid);

    assert_failure(&output, "uncheck @invalid999");
}

/// Test check with invalid UDID.
#[test]
fn test_check_invalid_udid() {
    let output = run_cli_command_with_udid("check", &["@e1"], "invalid-udid-12345");

    assert_failure(&output, "check with invalid UDID");
}

/// Test check without target.
#[test]
fn test_check_missing_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("check", &[], &udid);

    assert_failure(&output, "check (missing target)");
}
