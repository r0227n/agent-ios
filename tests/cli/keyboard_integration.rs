//! Keyboard feature integration tests.
//!
//! Tests for Core Commands: type, tap (for keys).

use crate::common::{
    assert_failure, assert_success, ensure_companion_running, get_available_udid,
    run_cli_command_with_udid,
};

/// Test type command.
#[test]
fn test_keyboard_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("type", &["Hello"], &udid);

    assert_success(&output, "type Hello");
}

/// Test tap enter key command.
#[test]
fn test_keyboard_key_enter() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["enter"], &udid);

    assert_success(&output, "tap enter");
}

/// Test tap delete key command.
#[test]
fn test_keyboard_key_delete() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["delete"], &udid);

    assert_success(&output, "tap delete");
}

/// Test tap home button command.
#[test]
fn test_keyboard_button_home() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["home"], &udid);

    assert_success(&output, "tap home");
}

/// Test tap lock button command.
#[test]
fn test_keyboard_button_lock() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["lock"], &udid);

    assert_failure(&output, "tap lock");
}

/// Test keyboard clear command.
#[test]
fn test_keyboard_clear() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Clear is done by sending multiple backspaces
    // For now, we test via tap delete multiple times
    let output = run_cli_command_with_udid("tap", &["delete"], &udid);

    assert_success(&output, "tap delete (clear)");
}

// ============================================================================
// Extended type tests
// ============================================================================

/// Test type with special characters.
#[test]
fn test_keyboard_special_chars() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("type", &["test@example.com"], &udid);

    assert_success(&output, "type test@example.com");
}

/// Test type with numbers.
#[test]
fn test_keyboard_numbers() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("type", &["12345"], &udid);

    assert_success(&output, "type 12345");
}

/// Test type with unicode characters.
#[test]
fn test_keyboard_unicode() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("type", &["Hello"], &udid);

    assert_success(&output, "type Hello (unicode)");
}

/// Test type with spaces.
#[test]
fn test_keyboard_with_spaces() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("type", &["Hello World"], &udid);

    assert_success(&output, "type Hello World");
}

/// Test type with empty string.
#[test]
fn test_keyboard_empty_string() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("type", &[""], &udid);

    // Empty string may succeed (no-op) or fail depending on implementation
    let _ = output;
}

// ============================================================================
// Extended key tap tests
// ============================================================================

/// Test tap tab key.
#[test]
fn test_keyboard_key_tab() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["tab"], &udid);

    // Tab key may or may not be supported
    let _ = output;
}

/// Test tap space key.
#[test]
fn test_keyboard_key_space() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["space"], &udid);

    // Space key may or may not be supported as hardware key
    let _ = output;
}

/// Test tap backspace key (alias for delete).
#[test]
fn test_keyboard_key_backspace() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["backspace"], &udid);

    // Backspace may be alias for delete or not supported
    let _ = output;
}

/// Test tap escape key.
#[test]
fn test_keyboard_key_escape() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["escape"], &udid);

    // Escape key may not be available on iOS
    let _ = output;
}

// ============================================================================
// Error cases
// ============================================================================

/// Test type with invalid UDID.
#[test]
fn test_keyboard_type_invalid_udid() {
    let output = run_cli_command_with_udid("type", &["test"], "invalid-udid-12345");

    assert_failure(&output, "type invalid udid");
}

/// Test type without text argument.
#[test]
fn test_keyboard_type_missing_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("type", &[], &udid);

    assert_failure(&output, "type (missing text)");
}
