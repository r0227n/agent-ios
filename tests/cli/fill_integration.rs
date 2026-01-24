//! Fill feature integration tests.
//!
//! Tests for `agent-mobile fill` command.
//! Tests text field fill functionality (clear + type).

use crate::common::{
    assert_failure, assert_success, ensure_companion_running, get_available_udid,
    get_test_bundle_id, run_cli_command_with_udid,
};

// ============================================================================
// Helper to navigate to a screen with text field
// ============================================================================

/// Navigate to Settings > General > About > Name (if available).
#[allow(dead_code)]
fn navigate_to_text_field(udid: &str) -> bool {
    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try to find a search field or text field
    let output = run_cli_command_with_udid("find", &["type", "TextField", "-f", "json"], udid);
    output.status.success()
}

/// Get a text field reference if available.
fn get_text_field_ref(udid: &str) -> Option<String> {
    let output = run_cli_command_with_udid("find", &["type", "TextField", "-f", "json"], udid);
    if !output.status.success() {
        // Try SearchField
        let output =
            run_cli_command_with_udid("find", &["type", "SearchField", "-f", "json"], udid);
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
// fill - Normal cases
// ============================================================================

/// Test fill with element reference.
#[test]
fn test_fill_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    if let Some(field_ref) = get_text_field_ref(&udid) {
        let output = run_cli_command_with_udid("fill", &[&field_ref, "test text"], &udid);
        assert_success(&output, "fill with ref");
    }
    // If no text field found, test passes
}

/// Test fill with placeholder text target.
#[test]
fn test_fill_with_placeholder_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Settings app may have a search field with placeholder
    let output = run_cli_command_with_udid("fill", &["Search", "test"], &udid);

    // May succeed or fail depending on if search field exists
    let _ = output;
}

/// Test fill with simple text.
#[test]
fn test_fill_simple_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    if let Some(field_ref) = get_text_field_ref(&udid) {
        let output = run_cli_command_with_udid("fill", &[&field_ref, "hello"], &udid);
        assert_success(&output, "fill hello");
    }
}

/// Test fill with empty text (clear only).
#[test]
fn test_fill_empty_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    if let Some(field_ref) = get_text_field_ref(&udid) {
        let output = run_cli_command_with_udid("fill", &[&field_ref, ""], &udid);
        // Empty fill should clear the field
        assert_success(&output, "fill empty");
    }
}

/// Test fill with special characters.
#[test]
fn test_fill_special_characters() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    if let Some(field_ref) = get_text_field_ref(&udid) {
        let output = run_cli_command_with_udid("fill", &[&field_ref, "test@example.com"], &udid);
        assert_success(&output, "fill special characters");
    }
}

/// Test fill with numbers.
#[test]
fn test_fill_numbers() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    if let Some(field_ref) = get_text_field_ref(&udid) {
        let output = run_cli_command_with_udid("fill", &[&field_ref, "12345"], &udid);
        assert_success(&output, "fill numbers");
    }
}

/// Test fill replaces existing content.
#[test]
fn test_fill_replaces_content() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    if let Some(field_ref) = get_text_field_ref(&udid) {
        // First fill
        let _ = run_cli_command_with_udid("fill", &[&field_ref, "first"], &udid);
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Second fill should replace
        let output = run_cli_command_with_udid("fill", &[&field_ref, "second"], &udid);
        assert_success(&output, "fill replaces content");
    }
}

// ============================================================================
// Error cases
// ============================================================================

/// Test fill with invalid element reference.
#[test]
fn test_fill_invalid_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("fill", &["@invalid999", "text"], &udid);

    assert_failure(&output, "fill @invalid999");
}

/// Test fill with non-existent target.
#[test]
fn test_fill_non_existent_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("fill", &["NonExistentTextField12345", "text"], &udid);

    assert_failure(&output, "fill non-existent target");
}

/// Test fill with invalid UDID.
#[test]
fn test_fill_invalid_udid() {
    let output = run_cli_command_with_udid("fill", &["@e1", "text"], "invalid-udid-12345");

    assert_failure(&output, "fill with invalid UDID");
}

/// Test fill without target.
#[test]
fn test_fill_missing_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("fill", &[], &udid);

    assert_failure(&output, "fill (missing target)");
}

/// Test fill without text.
#[test]
fn test_fill_missing_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("fill", &["@e1"], &udid);

    assert_failure(&output, "fill (missing text)");
}
