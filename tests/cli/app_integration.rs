//! App feature integration tests.
//!
//! Tests for `agent-mobile app` commands.

use crate::common::{
    assert_failure, assert_stdout_contains, assert_success, assert_valid_json,
    ensure_companion_running, get_available_udid, get_stdout, get_test_bundle_id,
    run_cli_command_with_timeout, run_cli_command_with_udid,
};
use std::time::Duration;

/// Test app list command.
#[test]
fn test_app_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("app", &["list"], &udid);

    assert_success(&output, "app list");
    assert_stdout_contains(&output, "Installed Apps");
}

/// Test app list with JSON output.
#[test]
fn test_app_list_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("app", &["list", "-f", "json"], &udid);

    assert_success(&output, "app list -f json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test app launch command.
#[test]
fn test_app_launch() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();
    let output = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);

    assert_success(&output, "app launch");
    assert_stdout_contains(&output, "Launched");
}

/// Test app launch with a best-effort fresh start.
#[test]
fn test_app_launch_fresh() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();
    let output = run_cli_command_with_udid("app", &["launch", &bundle_id, "--fresh"], &udid);

    assert_success(&output, "app launch --fresh");
    assert_stdout_contains(&output, "(fresh)");
}

/// Test app terminate command.
#[test]
fn test_app_terminate() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();

    // First launch the app
    let launch_output = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    assert_success(&launch_output, "app launch (setup)");

    // Small delay to ensure app is running
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Then terminate
    let output = run_cli_command_with_udid("app", &["terminate", &bundle_id], &udid);

    assert_success(&output, "app terminate");
    assert_stdout_contains(&output, "Terminated");
}

// ============================================================================
// Extended app list tests
// ============================================================================

/// Test app list shows Settings app.
#[test]
fn test_app_list_contains_settings() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("app", &["list", "-f", "json"], &udid);

    assert_success(&output, "app list -f json");
    let json = assert_valid_json(&output);

    // Settings app should be in the list
    if let Some(apps) = json.as_array() {
        let has_settings = apps.iter().any(|app| {
            app.get("bundle_id")
                .and_then(|b| b.as_str())
                .map(|b| b.contains("Preferences"))
                .unwrap_or(false)
        });
        assert!(has_settings, "Expected Settings app in list");
    }
}

/// Test app list with text format.
#[test]
fn test_app_list_text_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("app", &["list", "-f", "text"], &udid);

    assert_success(&output, "app list -f text");
    assert_stdout_contains(&output, "Installed Apps");
}

// ============================================================================
// Extended app launch tests
// ============================================================================

/// Test app launch with invalid bundle ID.
#[test]
fn test_app_launch_invalid_bundle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("app", &["launch", "com.invalid.bundle.id.12345"], &udid);

    assert_failure(&output, "app launch invalid bundle");
}

/// Test app launch multiple times (idempotent).
#[test]
fn test_app_launch_multiple() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();

    let output1 = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    assert_success(&output1, "app launch first");

    std::thread::sleep(std::time::Duration::from_millis(300));

    let output2 = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    assert_success(&output2, "app launch second");
}

/// Test app launch keeps the launched app in the runner context for the next command.
#[test]
fn test_app_launch_keeps_foreground_context_for_snapshot() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();
    let launch_output = run_cli_command_with_udid("app", &["launch", &bundle_id], &udid);
    assert_success(&launch_output, "app launch");

    let snapshot_output = run_cli_command_with_timeout(
        "snapshot",
        &["--no-scroll", "-f", "json", "--udid", &udid],
        Duration::from_secs(60),
    );
    assert_success(&snapshot_output, "snapshot after app launch");

    let stdout = get_stdout(&snapshot_output);
    assert!(
        !stdout.contains("XCUITestRunnerUITests-Runner")
            && !stdout.contains("\"label\": \"XCUITestRunner\""),
        "snapshot unexpectedly fell back to SpringBoard after launching {}:\n{}",
        bundle_id,
        stdout
    );
}

/// Test app launch --fresh keeps the launched app in the runner context for the next command.
#[test]
fn test_app_launch_fresh_keeps_foreground_context_for_snapshot() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();
    let launch_output = run_cli_command_with_udid("app", &["launch", &bundle_id, "--fresh"], &udid);
    assert_success(&launch_output, "app launch --fresh");

    let snapshot_output = run_cli_command_with_timeout(
        "snapshot",
        &["--no-scroll", "-f", "json", "--udid", &udid],
        Duration::from_secs(60),
    );
    assert_success(&snapshot_output, "snapshot after app launch --fresh");

    let stdout = get_stdout(&snapshot_output);
    assert!(
        !stdout.contains("XCUITestRunnerUITests-Runner")
            && !stdout.contains("\"label\": \"XCUITestRunner\""),
        "snapshot unexpectedly fell back to SpringBoard after fresh launching {}:\n{}",
        bundle_id,
        stdout
    );
}

// ============================================================================
// Extended app terminate tests
// ============================================================================

/// Test app terminate with invalid bundle ID.
#[test]
fn test_app_terminate_invalid_bundle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("app", &["terminate", "com.invalid.bundle.id.12345"], &udid);

    // May succeed (app not running) or fail depending on implementation
    let _ = output;
}

/// Test app terminate not running app.
#[test]
fn test_app_terminate_not_running() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let bundle_id = get_test_bundle_id();

    // Ensure app is not running
    let _ = run_cli_command_with_udid("app", &["terminate", &bundle_id], &udid);
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Try to terminate again
    let output = run_cli_command_with_udid("app", &["terminate", &bundle_id], &udid);

    // May succeed (idempotent) or fail
    let _ = output;
}

// ============================================================================
// app install tests
// ============================================================================

/// Test app install help exists.
#[test]
fn test_app_install_help() {
    let output = run_cli_command_with_udid("app", &["install", "--help"], "dummy");

    // --help should work without needing valid UDID
    assert!(
        output.status.success() || String::from_utf8_lossy(&output.stderr).contains("Usage"),
        "app install --help should show usage"
    );
}

/// Test app install with invalid path.
#[test]
fn test_app_install_invalid_path() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("app", &["install", "/nonexistent/app.ipa"], &udid);

    assert_failure(&output, "app install invalid path");
}

// ============================================================================
// app uninstall tests
// ============================================================================

/// Test app uninstall help exists.
#[test]
fn test_app_uninstall_help() {
    let output = run_cli_command_with_udid("app", &["uninstall", "--help"], "dummy");

    assert!(
        output.status.success() || String::from_utf8_lossy(&output.stderr).contains("Usage"),
        "app uninstall --help should show usage"
    );
}

/// Test app uninstall with invalid bundle.
#[test]
fn test_app_uninstall_invalid_bundle() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("app", &["uninstall", "com.invalid.bundle.id.12345"], &udid);

    // May succeed (app not installed) or fail
    let _ = output;
}

// ============================================================================
// Error cases
// ============================================================================

/// Test app list with invalid UDID.
#[test]
fn test_app_list_invalid_udid() {
    let output = run_cli_command_with_udid("app", &["list"], "invalid-udid-12345");

    assert_failure(&output, "app list invalid udid");
}

/// Test app launch with invalid UDID.
#[test]
fn test_app_launch_invalid_udid() {
    let bundle_id = get_test_bundle_id();
    let output = run_cli_command_with_udid("app", &["launch", &bundle_id], "invalid-udid-12345");

    assert_failure(&output, "app launch invalid udid");
}

// ============================================================================
// Platform-agnostic app tests (iOS and Android)
// ============================================================================

/// Test app list works on any platform.
#[test]
fn test_app_list_platform_agnostic() {
    use crate::common::{run_cli_command_with_device, DeviceIdentifier};

    // Try to get any available device
    let device = match DeviceIdentifier::get_any_available() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Skipping test: No iOS or Android device available");
            return;
        }
    };

    device.ensure_ready();
    let output = run_cli_command_with_device("app", &["list"], &device);

    assert_success(&output, "app list on any platform");
}

/// Test app launch works on any platform.
#[test]
fn test_app_launch_platform_agnostic() {
    use crate::common::{run_cli_command_with_device, DeviceIdentifier};

    let device = match DeviceIdentifier::get_any_available() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Skipping test: No device available");
            return;
        }
    };

    device.ensure_ready();
    let bundle_id = device.get_test_bundle_id();
    let output = run_cli_command_with_device("app", &["launch", &bundle_id], &device);

    assert_success(
        &output,
        &format!("app launch on {}", device.platform_name()),
    );
}

/// Test app terminate works on any platform.
#[test]
fn test_app_terminate_platform_agnostic() {
    use crate::common::{run_cli_command_with_device, DeviceIdentifier};

    let device = match DeviceIdentifier::get_any_available() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Skipping test: No device available");
            return;
        }
    };

    device.ensure_ready();
    let bundle_id = device.get_test_bundle_id();

    // Launch first
    let launch_output = run_cli_command_with_device("app", &["launch", &bundle_id], &device);
    assert_success(&launch_output, "app launch (setup)");

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Terminate
    let output = run_cli_command_with_device("app", &["terminate", &bundle_id], &device);
    assert_success(
        &output,
        &format!("app terminate on {}", device.platform_name()),
    );
}
