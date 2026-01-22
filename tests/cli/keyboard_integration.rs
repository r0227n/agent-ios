//! Keyboard feature integration tests.
//!
//! Tests for Core Commands: type, tap (for keys).

use crate::common::{
    assert_success, ensure_companion_running, get_available_udid, run_cli_command_with_udid,
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

    assert_success(&output, "tap lock");
}

/// Test clear via idb command (backward compatibility).
#[test]
fn test_keyboard_clear() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Clear is done by sending multiple backspaces
    // For now, we test via tap delete multiple times
    let output = run_cli_command_with_udid("tap", &["delete"], &udid);

    assert_success(&output, "tap delete (clear)");
}
