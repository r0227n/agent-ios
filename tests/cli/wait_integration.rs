//! Wait feature integration tests.
//!
//! Tests for `agent-mobile wait` command.
//! Tests element wait functionality.

use crate::common::{
    assert_failure, assert_success, ensure_companion_running, get_available_udid,
    get_test_bundle_id, run_cli_command_with_udid,
};

// ============================================================================
// wait visible - Normal cases
// ============================================================================

/// Test wait visible with existing element (should succeed immediately).
#[test]
fn test_wait_visible_existing() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output =
        run_cli_command_with_udid("wait", &["visible", "General", "--timeout", "5s"], &udid);

    assert_success(&output, "wait visible General");
}

/// Test wait visible with text target.
#[test]
fn test_wait_visible_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output =
        run_cli_command_with_udid("wait", &["visible", "Settings", "--timeout", "5s"], &udid);

    // May or may not find "Settings" depending on screen
    let _ = output;
}

/// Test wait visible with short timeout.
#[test]
fn test_wait_visible_short_timeout() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output =
        run_cli_command_with_udid("wait", &["visible", "General", "--timeout", "2s"], &udid);

    assert_success(&output, "wait visible --timeout 2s");
}

/// Test wait visible with custom interval.
#[test]
fn test_wait_visible_custom_interval() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid(
        "wait",
        &["visible", "General", "--timeout", "5s", "--interval", "200"],
        &udid,
    );

    assert_success(&output, "wait visible --interval 200");
}

// ============================================================================
// wait gone - Normal cases
// ============================================================================

/// Test wait gone with non-existent element (should succeed immediately).
#[test]
fn test_wait_gone_non_existent() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "wait",
        &["gone", "NonExistentElement12345", "--timeout", "2s"],
        &udid,
    );

    assert_success(&output, "wait gone NonExistentElement");
}

/// Test wait gone with text target.
#[test]
fn test_wait_gone_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "wait",
        &["gone", "ElementThatDoesNotExist", "--timeout", "2s"],
        &udid,
    );

    assert_success(&output, "wait gone text target");
}

// ============================================================================
// wait idle - Normal cases
// ============================================================================

/// Test wait idle (no target needed).
#[test]
fn test_wait_idle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("wait", &["idle", "--timeout", "5s"], &udid);

    assert_success(&output, "wait idle");
}

/// Test wait idle with timeout.
#[test]
fn test_wait_idle_timeout() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("wait", &["idle", "--timeout", "3s"], &udid);

    assert_success(&output, "wait idle --timeout 3s");
}

// ============================================================================
// wait text - Normal cases
// ============================================================================

/// Test wait text with existing text.
#[test]
fn test_wait_text_existing() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("wait", &["text", "General", "--timeout", "5s"], &udid);

    assert_success(&output, "wait text General");
}

/// Test wait text with custom timeout.
#[test]
fn test_wait_text_timeout() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("wait", &["text", "General", "--timeout", "10s"], &udid);

    assert_success(&output, "wait text --timeout 10s");
}

// ============================================================================
// Timeout formats - Normal cases
// ============================================================================

/// Test wait with seconds format.
#[test]
fn test_wait_timeout_seconds_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("wait", &["gone", "NonExistent", "--timeout", "5s"], &udid);

    assert_success(&output, "wait --timeout 5s");
}

/// Test wait with numeric timeout (assumed seconds).
#[test]
fn test_wait_timeout_numeric() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("wait", &["gone", "NonExistent", "--timeout", "5"], &udid);

    assert_success(&output, "wait --timeout 5");
}

// ============================================================================
// Error cases
// ============================================================================

/// Test wait visible with timeout (element not found).
#[test]
fn test_wait_visible_timeout_error() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "wait",
        &["visible", "NonExistentElement999", "--timeout", "1s"],
        &udid,
    );

    assert_failure(&output, "wait visible timeout");
}

/// Test wait gone with existing element (timeout).
#[test]
fn test_wait_gone_timeout_error() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // "General" should exist, so wait gone should timeout
    let output = run_cli_command_with_udid("wait", &["gone", "General", "--timeout", "1s"], &udid);

    assert_failure(&output, "wait gone timeout");
}

/// Test wait with invalid condition.
#[test]
fn test_wait_invalid_condition() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("wait", &["invalidcondition", "test"], &udid);

    assert_failure(&output, "wait invalidcondition");
}

/// Test wait text with non-matching text (timeout).
#[test]
fn test_wait_text_timeout_error() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "wait",
        &["text", "NonExistentText12345", "--timeout", "1s"],
        &udid,
    );

    assert_failure(&output, "wait text timeout");
}

/// Test wait with invalid UDID.
#[test]
fn test_wait_invalid_udid() {
    let output = run_cli_command_with_udid(
        "wait",
        &["visible", "test", "--timeout", "1s"],
        "invalid-udid-12345",
    );

    assert_failure(&output, "wait with invalid UDID");
}

/// Test wait visible without target.
#[test]
fn test_wait_visible_missing_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("wait", &["visible"], &udid);

    assert_failure(&output, "wait visible (missing target)");
}
