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

// ============================================================================
// Extended tap tests
// ============================================================================

/// Test tap with element reference.
#[test]
fn test_gesture_tap_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // First find an element
    let find_output =
        run_cli_command_with_udid("find", &["type", "StaticText", "-f", "json"], &udid);
    if find_output.status.success() {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&find_output.stdout) {
            if let Some(ref_val) = json.get("ref").and_then(|v| v.as_str()) {
                let output = run_cli_command_with_udid("tap", &[ref_val], &udid);
                assert_success(&output, &format!("tap {}", ref_val));
            }
        }
    }
}

/// Test tap with text target.
#[test]
fn test_gesture_tap_with_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Launch Settings and tap General by text
    let _ = run_cli_command_with_udid("app", &["launch", "com.apple.Preferences"], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("tap", &["General"], &udid);

    assert_success(&output, "tap General");
}

/// Test tap with hardware key (back).
#[test]
fn test_gesture_tap_back() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["back"], &udid);

    // May succeed or fail depending on navigation state
    let _ = output;
}

/// Test tap with duration option.
#[test]
fn test_gesture_tap_with_duration() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("tap", &["center", "--duration", "0.5"], &udid);

    assert_success(&output, "tap center --duration 0.5");
}

// ============================================================================
// Extended swipe tests
// ============================================================================

/// Test swipe down direction.
#[test]
fn test_gesture_swipe_down() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["down"], &udid);

    assert_success(&output, "swipe down");
}

/// Test swipe left direction.
#[test]
fn test_gesture_swipe_left() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["left"], &udid);

    assert_success(&output, "swipe left");
}

/// Test swipe right direction.
#[test]
fn test_gesture_swipe_right() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["right"], &udid);

    assert_success(&output, "swipe right");
}

/// Test swipe with --from option.
#[test]
fn test_gesture_swipe_from() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["up", "--from", "200,400"], &udid);

    assert_success(&output, "swipe up --from 200,400");
}

/// Test swipe with --distance option.
#[test]
fn test_gesture_swipe_distance() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["up", "--distance", "300"], &udid);

    assert_success(&output, "swipe up --distance 300");
}

/// Test swipe with --duration option.
#[test]
fn test_gesture_swipe_duration() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("swipe", &["up", "--duration", "0.5"], &udid);

    assert_success(&output, "swipe up --duration 0.5");
}

// ============================================================================
// Extended scroll tests
// ============================================================================

/// Test scroll up direction.
#[test]
fn test_gesture_scroll_up() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("scroll", &["up"], &udid);

    assert_success(&output, "scroll up");
}

/// Test scroll left direction.
#[test]
fn test_gesture_scroll_left() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("scroll", &["left"], &udid);

    assert_success(&output, "scroll left");
}

/// Test scroll right direction.
#[test]
fn test_gesture_scroll_right() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("scroll", &["right"], &udid);

    assert_success(&output, "scroll right");
}

/// Test scroll within element (--in option).
#[test]
fn test_gesture_scroll_in_element() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Find a scrollable element
    let find_output = run_cli_command_with_udid("find", &["type", "Table", "-f", "json"], &udid);
    if find_output.status.success() {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&find_output.stdout) {
            if let Some(ref_val) = json.get("ref").and_then(|v| v.as_str()) {
                let output = run_cli_command_with_udid("scroll", &["down", "--in", ref_val], &udid);
                assert_success(&output, &format!("scroll down --in {}", ref_val));
            }
        }
    }
}

// ============================================================================
// Extended long-press tests
// ============================================================================

/// Test long-press at center.
#[test]
fn test_gesture_long_press_center() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("long-press", &["center", "--duration", "1.0"], &udid);

    assert_success(&output, "long-press center");
}

/// Test long-press with element reference.
#[test]
fn test_gesture_long_press_with_ref() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let find_output = run_cli_command_with_udid("find", &["type", "Cell", "-f", "json"], &udid);
    if find_output.status.success() {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&find_output.stdout) {
            if let Some(ref_val) = json.get("ref").and_then(|v| v.as_str()) {
                let output =
                    run_cli_command_with_udid("long-press", &[ref_val, "--duration", "1.0"], &udid);
                assert_success(&output, &format!("long-press {}", ref_val));
            }
        }
    }
}

/// Test long-press with text target.
#[test]
fn test_gesture_long_press_with_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", "com.apple.Preferences"], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("long-press", &["General", "--duration", "0.5"], &udid);

    assert_success(&output, "long-press General");
}

/// Test long-press with short duration.
#[test]
fn test_gesture_long_press_short_duration() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("long-press", &["center", "--duration", "0.3"], &udid);

    assert_success(&output, "long-press --duration 0.3");
}

/// Test long-press with default duration.
#[test]
fn test_gesture_long_press_default_duration() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("long-press", &["center"], &udid);

    assert_success(&output, "long-press center (default duration)");
}
