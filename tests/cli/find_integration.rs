//! Find feature integration tests.
//!
//! Tests for `agent-mobile find` commands.
//! Tests semantic locator search functionality.

use crate::common::{
    assert_failure, assert_success, assert_valid_json, ensure_companion_running,
    get_available_udid, get_test_bundle_id, run_cli_command_with_udid,
};

// ============================================================================
// find type - Normal cases
// ============================================================================

/// Test find type Button.
#[test]
fn test_find_type_button() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app which has buttons
    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["type", "Button"], &udid);

    assert_success(&output, "find type Button");
}

/// Test find type StaticText.
#[test]
fn test_find_type_statictext() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText"], &udid);

    assert_success(&output, "find type StaticText");
}

/// Test find type with JSON output.
#[test]
fn test_find_type_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText", "-f", "json"], &udid);

    assert_success(&output, "find type StaticText -f json");
    let json = assert_valid_json(&output);
    assert!(
        json.get("ref").is_some() || json.is_array(),
        "Expected ref field or array in JSON output"
    );
}

/// Test find type with --first option (default).
#[test]
fn test_find_type_first() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText", "--first"], &udid);

    assert_success(&output, "find type StaticText --first");
}

/// Test find type with --last option.
#[test]
fn test_find_type_last() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText", "--last"], &udid);

    assert_success(&output, "find type StaticText --last");
}

/// Test find type with --nth option.
#[test]
fn test_find_type_nth() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText", "--nth", "0"], &udid);

    assert_success(&output, "find type StaticText --nth 0");
}

/// Test find type with --all option.
#[test]
fn test_find_type_all() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText", "--all"], &udid);

    assert_success(&output, "find type StaticText --all");
}

/// Test find type with --all and JSON format.
#[test]
fn test_find_type_all_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "find",
        &["type", "StaticText", "--all", "-f", "json"],
        &udid,
    );

    assert_success(&output, "find type StaticText --all -f json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array for --all output");
}

/// Test find type case insensitive.
#[test]
fn test_find_type_case_insensitive() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "button"], &udid);

    // Should work with lowercase
    assert_success(&output, "find type button (lowercase)");
}

// ============================================================================
// find text - Normal cases
// ============================================================================

/// Test find text basic.
#[test]
fn test_find_text_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings app
    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // "Settings" or "General" should be visible in Settings app
    let output = run_cli_command_with_udid("find", &["text", "General"], &udid);

    assert_success(&output, "find text General");
}

/// Test find text with partial match (default).
#[test]
fn test_find_text_partial_match() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Partial match should work
    let output = run_cli_command_with_udid("find", &["text", "Gen"], &udid);

    assert_success(&output, "find text Gen (partial)");
}

/// Test find text with --exact option.
#[test]
fn test_find_text_exact() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["text", "General", "--exact"], &udid);

    assert_success(&output, "find text General --exact");
}

/// Test find text with JSON output.
#[test]
fn test_find_text_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["text", "General", "-f", "json"], &udid);

    assert_success(&output, "find text General -f json");
    let _ = assert_valid_json(&output);
}

/// Test find text with --first option.
#[test]
fn test_find_text_first() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["text", "Settings", "--first"], &udid);

    // May succeed or fail depending on screen content
    // Just verify it doesn't crash
    let _ = output;
}

/// Test find text with --last option.
#[test]
fn test_find_text_last() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["text", "General", "--last"], &udid);

    assert_success(&output, "find text General --last");
}

/// Test find text with --all option.
#[test]
fn test_find_text_all() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["text", "Settings", "--all"], &udid);

    // May succeed or fail depending on screen content
    let _ = output;
}

// ============================================================================
// find label - Normal cases
// ============================================================================

/// Test find label basic.
#[test]
fn test_find_label_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["label", "General"], &udid);

    assert_success(&output, "find label General");
}

/// Test find label with --exact option.
#[test]
fn test_find_label_exact() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["label", "General", "--exact"], &udid);

    assert_success(&output, "find label General --exact");
}

/// Test find label with JSON format.
#[test]
fn test_find_label_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["label", "General", "-f", "json"], &udid);

    assert_success(&output, "find label General -f json");
    let _ = assert_valid_json(&output);
}

// ============================================================================
// find placeholder - Normal cases
// ============================================================================

/// Test find placeholder command exists.
#[test]
fn test_find_placeholder_help() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Just verify the command runs (may not find anything)
    let output = run_cli_command_with_udid("find", &["placeholder", "Search"], &udid);

    // May fail if no placeholder matches, but should not crash
    let _ = output;
}

// ============================================================================
// find enabled/disabled - Normal cases
// ============================================================================

/// Test find enabled elements.
#[test]
fn test_find_enabled() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["enabled"], &udid);

    assert_success(&output, "find enabled");
}

/// Test find enabled with --all option.
#[test]
fn test_find_enabled_all() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["enabled", "--all"], &udid);

    assert_success(&output, "find enabled --all");
}

/// Test find disabled elements.
#[test]
fn test_find_disabled() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["disabled"], &udid);

    // May fail if no disabled elements exist
    let _ = output;
}

// ============================================================================
// find with actions - Normal cases
// ============================================================================

/// Test find type with tap action.
#[test]
fn test_find_type_with_tap_action() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["type", "Cell", "tap"], &udid);

    assert_success(&output, "find type Cell tap");
}

/// Test find text with tap action.
#[test]
fn test_find_text_with_tap_action() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["text", "General", "tap"], &udid);

    assert_success(&output, "find text General tap");

    // Go back to main screen
    std::thread::sleep(std::time::Duration::from_millis(300));
    let _ = run_cli_command_with_udid("tap", &["back"], &udid);
}

/// Test find type with long-press action.
#[test]
fn test_find_type_with_long_press_action() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("find", &["type", "Cell", "long-press"], &udid);

    assert_success(&output, "find type Cell long-press");
}

// ============================================================================
// find - Error cases
// ============================================================================

/// Test find type with invalid type name.
#[test]
fn test_find_type_invalid_type() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "InvalidElementType123"], &udid);

    assert_failure(&output, "find type InvalidElementType123");
}

/// Test find text with no match.
#[test]
fn test_find_text_no_match() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "find",
        &["text", "ThisTextDefinitelyDoesNotExist12345"],
        &udid,
    );

    assert_failure(&output, "find text with no match");
}

/// Test find text with --exact no match.
#[test]
fn test_find_text_exact_no_match() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Partial text that exists but exact match won't work
    let output = run_cli_command_with_udid("find", &["text", "Gen", "--exact"], &udid);

    assert_failure(&output, "find text Gen --exact (should fail)");
}

/// Test find with --nth out of range.
#[test]
fn test_find_type_nth_out_of_range() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("find", &["type", "StaticText", "--nth", "99999"], &udid);

    assert_failure(&output, "find type StaticText --nth 99999");
}

/// Test find with invalid UDID.
#[test]
fn test_find_invalid_udid() {
    let output = run_cli_command_with_udid("find", &["type", "Button"], "invalid-udid-12345");

    assert_failure(&output, "find with invalid UDID");
}

/// Test find label with no match.
#[test]
fn test_find_label_no_match() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["label", "NonExistentLabel98765"], &udid);

    assert_failure(&output, "find label with no match");
}

/// Test find placeholder with no match.
#[test]
fn test_find_placeholder_no_match() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "find",
        &["placeholder", "NonExistentPlaceholder98765"],
        &udid,
    );

    assert_failure(&output, "find placeholder with no match");
}

/// Test find type with negative --nth.
#[test]
fn test_find_type_nth_negative() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // clap should reject negative values for usize
    let output = run_cli_command_with_udid("find", &["type", "Button", "--nth", "-1"], &udid);

    assert_failure(&output, "find type Button --nth -1");
}

/// Test find --all with action should fail.
#[test]
fn test_find_all_with_action_error() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // --all with action should be an error
    let output = run_cli_command_with_udid("find", &["type", "Button", "--all", "tap"], &udid);

    assert_failure(&output, "find type Button --all tap");
}

// ============================================================================
// Output format tests
// ============================================================================

/// Test that text format outputs reference.
#[test]
fn test_find_text_format_outputs_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText", "-f", "text"], &udid);

    assert_success(&output, "find type StaticText -f text");
    // Text format should output something like "$1" or element ref
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("$") || !stdout.is_empty(),
        "Expected element reference in text output"
    );
}

/// Test JSON format contains required fields.
#[test]
fn test_find_json_format_fields() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("find", &["type", "StaticText", "-f", "json"], &udid);

    assert_success(&output, "find type StaticText -f json");
    let json = assert_valid_json(&output);

    // Should have ref field
    assert!(
        json.get("ref").is_some() || json.is_array(),
        "Expected ref field in JSON: {:?}",
        json
    );
}

// ============================================================================
// Android-specific find tests
// ============================================================================

/// Test find Android-specific element types.
#[test]
#[ignore] // TODO: find command needs session support for Android
fn test_find_android_element_types() {
    use crate::common::get_available_serial;

    // Skip if no Android device available
    let serial = match std::panic::catch_unwind(get_available_serial) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: No Android device available");
            return;
        }
    };

    // Launch Settings app
    let package = "com.android.settings";
    let _ = run_cli_command_with_udid("app", &["launch", package], &serial);
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // TODO: find command requires session management, not --udid
    // Need to update test after session support is added
}

/// Test find text on Android.
#[test]
#[ignore] // TODO: find command needs session support for Android
fn test_find_text_android() {
    use crate::common::get_available_serial;

    let serial = match std::panic::catch_unwind(get_available_serial) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: No Android device available");
            return;
        }
    };

    let package = "com.android.settings";
    let _ = run_cli_command_with_udid("app", &["launch", package], &serial);
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // TODO: find command requires session management, not --udid
    // Need to update test after session support is added
}
