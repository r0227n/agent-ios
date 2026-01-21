//! Accessibility feature integration tests.
//!
//! Tests for `agent-mobile accessibility` commands.

use crate::common::{
    assert_stdout_contains, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, run_cli_command_with_udid,
};

/// Test accessibility --audit command.
#[test]
fn test_accessibility_audit() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("accessibility", &["--audit"], &udid);

    assert_success(&output, "accessibility --audit");
    assert_stdout_contains(&output, "Accessibility Audit Results");
}

/// Test accessibility --audit with JSON output.
#[test]
fn test_accessibility_audit_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("accessibility", &["--audit", "-o", "json"], &udid);

    assert_success(&output, "accessibility --audit -o json");
    let json = assert_valid_json(&output);

    // Check expected JSON structure
    assert!(
        json.get("total_elements").is_some(),
        "Expected 'total_elements' field in JSON"
    );
    assert!(
        json.get("issues").is_some(),
        "Expected 'issues' field in JSON"
    );
    assert!(
        json.get("score").is_some(),
        "Expected 'score' field in JSON"
    );
}

/// Test accessibility default action (--audit).
#[test]
fn test_accessibility_default() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Without specifying --audit, it should still run audit
    let output = run_cli_command_with_udid("accessibility", &[], &udid);

    assert_success(&output, "accessibility (default)");
    assert_stdout_contains(&output, "Accessibility Audit Results");
}
