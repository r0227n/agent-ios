//! Gesture feature integration tests.
//!
//! Tests for `agent-mobile gesture` commands.

use crate::common::{
    assert_success, ensure_companion_running, get_available_udid, run_cli_command_with_udid,
};

/// Test gesture tap at screen center.
#[test]
fn test_gesture_tap_center() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("gesture", &["tap"], &udid);

    assert_success(&output, "gesture tap (center)");
}

/// Test gesture tap with coordinates.
#[test]
fn test_gesture_tap_coords() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("gesture", &["tap", "100,200"], &udid);

    assert_success(&output, "gesture tap 100,200");
}

/// Test gesture swipe up direction.
#[test]
fn test_gesture_swipe_up() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("gesture", &["swipe", "up"], &udid);

    assert_success(&output, "gesture swipe up");
}

/// Test gesture swipe with custom coordinates.
#[test]
fn test_gesture_swipe_coords() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("gesture", &["swipe", "100,100,200,200"], &udid);

    assert_success(&output, "gesture swipe 100,100,200,200");
}

/// Test gesture scroll down direction.
#[test]
fn test_gesture_scroll_down() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("gesture", &["scroll", "down"], &udid);

    assert_success(&output, "gesture scroll down");
}

/// Test gesture long-press with coordinates.
#[test]
fn test_gesture_long_press() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("gesture", &["long-press", "100,200"], &udid);

    assert_success(&output, "gesture long-press 100,200");
}
