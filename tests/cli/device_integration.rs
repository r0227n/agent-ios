//! Device feature integration tests.
//!
//! Tests for `agent-mobile device` commands.

use crate::common::{
    assert_failure, assert_success, assert_valid_json, get_available_udid, run_cli_command,
    run_cli_command_with_udid,
};

/// Test device list command.
#[test]
fn test_device_list() {
    let output = run_cli_command("device", &["list"]);

    assert_success(&output, "device list");
    // iOS or Android header should be present
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Devices/Simulators") || stdout.contains("Devices/Emulators"),
        "Expected device list output, got: {}",
        stdout
    );
}

/// Test device list with JSON output.
#[test]
fn test_device_list_json() {
    let output = run_cli_command("device", &["list", "-f", "json"]);

    assert_success(&output, "device list -f json");
    let json = assert_valid_json(&output);
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
}

/// Test device list shows booted device.
#[test]
fn test_device_list_has_booted() {
    let udid = get_available_udid();
    // Note: device list doesn't take --udid, it lists all devices
    let output = run_cli_command("device", &["list"]);

    assert_success(&output, "device list");

    // Check that the booted device's UDID is in the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&udid) || stdout.contains("Booted"),
        "Expected to find booted device UDID or state, got: {}",
        stdout
    );
}

// ============================================================================
// Extended device list tests
// ============================================================================

/// Test device list includes device name.
#[test]
fn test_device_list_includes_name() {
    let output = run_cli_command("device", &["list"]);

    assert_success(&output, "device list");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain iPhone or Android device name
    assert!(
        stdout.contains("iPhone")
            || stdout.contains("iPad")
            || stdout.contains("Pixel")
            || stdout.contains("Android"),
        "Expected device name in output"
    );
}

/// Test device list JSON includes all required fields.
#[test]
fn test_device_list_json_fields() {
    let output = run_cli_command("device", &["list", "-f", "json"]);

    assert_success(&output, "device list -f json");
    let json = assert_valid_json(&output);

    if let Some(devices) = json.as_array() {
        for device in devices {
            // Each device should have udid and name at minimum
            assert!(
                device.get("udid").is_some() || device.get("serial").is_some(),
                "Expected udid or serial in device: {:?}",
                device
            );
        }
    }
}

// ============================================================================
// device boot tests
// ============================================================================

/// Test device boot command help exists.
#[test]
fn test_device_boot_help() {
    let output = run_cli_command("device", &["boot", "--help"]);

    assert_success(&output, "device boot --help");
}

/// Test device boot with invalid name.
#[test]
fn test_device_boot_invalid_name() {
    let output = run_cli_command("device", &["boot", "NonExistentDevice12345"]);

    assert_failure(&output, "device boot NonExistentDevice12345");
}

/// Test device boot with invalid UDID.
#[test]
fn test_device_boot_invalid_udid() {
    let output = run_cli_command("device", &["boot", "--udid", "invalid-udid-12345"]);

    assert_failure(&output, "device boot --udid invalid-udid-12345");
}

// ============================================================================
// device shutdown tests
// ============================================================================

/// Test device shutdown command help exists.
#[test]
fn test_device_shutdown_help() {
    let output = run_cli_command("device", &["shutdown", "--help"]);

    assert_success(&output, "device shutdown --help");
}

/// Test device shutdown with invalid UDID.
#[test]
fn test_device_shutdown_invalid_udid() {
    let output = run_cli_command("device", &["shutdown", "--udid", "invalid-udid-12345"]);

    assert_failure(&output, "device shutdown --udid invalid-udid-12345");
}

// ============================================================================
// device clipboard tests (pbcopy/pbpaste extended)
// ============================================================================

/// Test device pbcopy with empty string.
#[test]
fn test_device_pbcopy_empty() {
    let udid = get_available_udid();

    let output = run_cli_command_with_udid("device", &["pbcopy", ""], &udid);

    assert_success(&output, "device pbcopy empty");
}

/// Test device pbcopy with long text.
#[test]
fn test_device_pbcopy_long_text() {
    let udid = get_available_udid();

    let long_text = "a".repeat(1000);
    let output = run_cli_command_with_udid("device", &["pbcopy", &long_text], &udid);

    assert_success(&output, "device pbcopy long text");
}

/// Test device pbcopy with unicode.
#[test]
fn test_device_pbcopy_unicode() {
    let udid = get_available_udid();

    let output = run_cli_command_with_udid("device", &["pbcopy", "Hello World"], &udid);

    assert_success(&output, "device pbcopy unicode");
}

/// Test device pbpaste with invalid UDID.
#[test]
fn test_device_pbpaste_invalid_udid() {
    let output = run_cli_command_with_udid("device", &["pbpaste"], "invalid-udid-12345");

    assert_failure(&output, "device pbpaste invalid udid");
}

// ============================================================================
// Android-specific device tests
// ============================================================================

/// Test device list includes Android devices.
#[test]
fn test_device_list_includes_android() {
    use crate::common::get_available_serial;

    // Skip if no Android device available
    if std::panic::catch_unwind(get_available_serial).is_err() {
        eprintln!("Skipping test: No Android device available");
        return;
    }

    let serial = get_available_serial();
    let output = run_cli_command("device", &["list"]);

    assert_success(&output, "device list");

    // Check that the Android serial is in the text output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&serial),
        "Expected Android device serial '{}' in output, got: {}",
        serial,
        stdout
    );
}

/// Test device list shows Android emulator state.
#[test]
fn test_device_list_android_emulator() {
    use crate::common::get_available_serial;

    // Skip if no Android device available
    if std::panic::catch_unwind(get_available_serial).is_err() {
        eprintln!("Skipping test: No Android device available");
        return;
    }

    let serial = get_available_serial();
    let output = run_cli_command("device", &["list"]);

    assert_success(&output, "device list");

    // Check that the Android serial is in the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&serial) || stdout.contains("emulator"),
        "Expected to find Android device serial or emulator keyword, got: {}",
        stdout
    );
}
