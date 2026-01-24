//! Gesture feature integration tests.
//!
//! Tests for Core Commands: tap, swipe, scroll.

use crate::common::{
    assert_success, ensure_companion_running, get_available_udid, run_cli_command_with_udid,
};

/// Test tap at screen center.
#[test]
fn test_gesture_tap_center() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["center"], &udid);

    assert_success(&output, "tap center");
}

/// Test tap with coordinates.
#[test]
fn test_gesture_tap_coords() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["100,200"], &udid);

    assert_success(&output, "tap 100,200");
}

/// Test swipe up direction.
#[test]
fn test_gesture_swipe_up() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["up"], &udid);

    assert_success(&output, "swipe up");
}

/// Test swipe with custom coordinates.
#[test]
fn test_gesture_swipe_coords() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["100,100,200,200"], &udid);

    assert_success(&output, "swipe 100,100,200,200");
}

/// Test scroll down direction.
#[test]
fn test_gesture_scroll_down() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("scroll", &["down"], &udid);

    assert_success(&output, "scroll down");
}

/// Test long-press command.
#[test]
fn test_gesture_long_press() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("long-press", &["100,200", "--duration", "1.0"], &udid);

    assert_success(&output, "long-press 100,200 --duration 1.0");
}
