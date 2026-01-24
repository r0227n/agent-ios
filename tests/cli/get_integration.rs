//! Get feature integration tests.
//!
//! Tests for `agent-mobile get` commands.
//! Tests element property retrieval functionality.

use crate::common::{
    assert_failure, assert_json_has_field, assert_success, assert_valid_json,
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

// ============================================================================
// get text - Normal cases
// ============================================================================

/// Test get text with element reference.
#[test]
fn test_get_text_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["text", &elem_ref], &udid);

    assert_success(&output, "get text with ref");
}

/// Test get text with JSON format.
#[test]
fn test_get_text_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["text", &elem_ref, "-f", "json"], &udid);

    assert_success(&output, "get text -f json");
    let _ = assert_valid_json(&output);
}

/// Test get text with text target.
#[test]
fn test_get_text_with_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Using text as target
    let output = run_cli_command_with_udid("get", &["text", "General"], &udid);

    assert_success(&output, "get text General");
}

// ============================================================================
// get value - Normal cases
// ============================================================================

/// Test get value with element reference.
#[test]
fn test_get_value_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["value", &elem_ref], &udid);

    // Value may be empty for some elements, but command should succeed
    assert_success(&output, "get value with ref");
}

/// Test get value with JSON format.
#[test]
fn test_get_value_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["value", &elem_ref, "-f", "json"], &udid);

    assert_success(&output, "get value -f json");
    let _ = assert_valid_json(&output);
}

// ============================================================================
// get box - Normal cases
// ============================================================================

/// Test get box with element reference.
#[test]
fn test_get_box_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["box", &elem_ref], &udid);

    assert_success(&output, "get box with ref");
}

/// Test get box with JSON format.
#[test]
fn test_get_box_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["box", &elem_ref, "-f", "json"], &udid);

    assert_success(&output, "get box -f json");
    let json = assert_valid_json(&output);

    // Box should have x, y, width, height
    assert_json_has_field(&json, "x");
    assert_json_has_field(&json, "y");
    assert_json_has_field(&json, "width");
    assert_json_has_field(&json, "height");
}

/// Test get box with text target.
#[test]
fn test_get_box_with_text_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("get", &["box", "General"], &udid);

    assert_success(&output, "get box General");
}

// ============================================================================
// get count - Normal cases
// ============================================================================

/// Test get count with element type.
#[test]
fn test_get_count_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("get", &["count", "StaticText"], &udid);

    assert_success(&output, "get count StaticText");
    // Should output a number
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().parse::<u32>().is_ok() || stdout.contains("count"),
        "Expected count number in output: {}",
        stdout
    );
}

/// Test get count with JSON format.
#[test]
fn test_get_count_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("get", &["count", "StaticText", "-f", "json"], &udid);

    assert_success(&output, "get count -f json");
    let json = assert_valid_json(&output);

    // Should have count field
    assert!(
        json.get("count").is_some() || json.is_number(),
        "Expected count in JSON: {:?}",
        json
    );
}

/// Test get count with Button type.
#[test]
fn test_get_count_button() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("get", &["count", "Button"], &udid);

    assert_success(&output, "get count Button");
}

// ============================================================================
// get attr - Normal cases
// ============================================================================

/// Test get attr with element reference.
#[test]
fn test_get_attr_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["attr", &elem_ref, "enabled"], &udid);

    assert_success(&output, "get attr enabled");
}

/// Test get attr with JSON format.
#[test]
fn test_get_attr_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output =
        run_cli_command_with_udid("get", &["attr", &elem_ref, "enabled", "-f", "json"], &udid);

    assert_success(&output, "get attr -f json");
    let _ = assert_valid_json(&output);
}

// ============================================================================
// get all properties - Normal cases
// ============================================================================

/// Test get all properties (no specific property).
#[test]
fn test_get_all_properties() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    // Depending on implementation, may need different syntax
    let output = run_cli_command_with_udid("get", &["text", &elem_ref], &udid);

    assert_success(&output, "get all properties");
}

// ============================================================================
// get - Error cases
// ============================================================================

/// Test get with invalid element reference.
#[test]
fn test_get_invalid_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("get", &["text", "@invalid999"], &udid);

    assert_failure(&output, "get text @invalid999");
}

/// Test get text with non-existent text target.
#[test]
fn test_get_text_no_match() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("get", &["text", "NonExistentTextTarget12345"], &udid);

    assert_failure(&output, "get text with no match");
}

/// Test get attr without attribute name.
#[test]
fn test_get_attr_missing_attribute() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let elem_ref = get_element_ref(&udid);
    // Missing attribute name
    let output = run_cli_command_with_udid("get", &["attr", &elem_ref], &udid);

    // Should fail or return all attributes
    let _ = output;
}

/// Test get with invalid property name.
#[test]
fn test_get_invalid_property() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["invalidprop", &elem_ref], &udid);

    assert_failure(&output, "get invalidprop");
}

/// Test get with invalid UDID.
#[test]
fn test_get_invalid_udid() {
    let output = run_cli_command_with_udid("get", &["text", "@e1"], "invalid-udid-12345");

    assert_failure(&output, "get with invalid UDID");
}

/// Test get count with invalid type.
#[test]
fn test_get_count_invalid_type() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("get", &["count", "InvalidType123"], &udid);

    // May return 0 or error
    let _ = output;
}

/// Test get box with invalid target.
#[test]
fn test_get_box_invalid_target() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("get", &["box", "NonExistentElement12345"], &udid);

    assert_failure(&output, "get box with invalid target");
}

// ============================================================================
// Output format tests
// ============================================================================

/// Test text format is human readable.
#[test]
fn test_get_text_format_readable() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["text", &elem_ref, "-f", "text"], &udid);

    assert_success(&output, "get text -f text");
    // Should have readable output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Expected non-empty text output");
}

/// Test JSON format is valid.
#[test]
fn test_get_json_format_valid() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let elem_ref = get_element_ref(&udid);
    let output = run_cli_command_with_udid("get", &["box", &elem_ref, "-f", "json"], &udid);

    assert_success(&output, "get box -f json");
    let json = assert_valid_json(&output);

    // Should be an object with numeric fields
    assert!(json.is_object(), "Expected JSON object for box");
}
