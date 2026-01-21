//! Keyboard feature integration tests.
//!
//! Tests for `agent-mobile keyboard` commands.

use crate::common::{
    assert_success, ensure_companion_running, get_available_udid, run_cli_command_with_udid,
};

/// Test keyboard text command.
#[test]
fn test_keyboard_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("keyboard", &["text", "Hello"], &udid);

    assert_success(&output, "keyboard text Hello");
}

/// Test keyboard key enter command.
#[test]
fn test_keyboard_key_enter() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("keyboard", &["key", "enter"], &udid);

    assert_success(&output, "keyboard key enter");
}

/// Test keyboard key delete command.
#[test]
fn test_keyboard_key_delete() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("keyboard", &["key", "delete"], &udid);

    assert_success(&output, "keyboard key delete");
}

/// Test keyboard button home command.
#[test]
fn test_keyboard_button_home() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("keyboard", &["button", "home"], &udid);

    assert_success(&output, "keyboard button home");
}

/// Test keyboard button lock command.
#[test]
fn test_keyboard_button_lock() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("keyboard", &["button", "lock"], &udid);

    assert_success(&output, "keyboard button lock");
}

/// Test keyboard clear command.
#[test]
fn test_keyboard_clear() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("keyboard", &["clear"], &udid);

    assert_success(&output, "keyboard clear");
}
