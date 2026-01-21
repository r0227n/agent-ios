//! Device feature integration tests.
//!
//! Tests for `agent-mobile device` commands.

use crate::common::{assert_success, assert_valid_json, get_available_udid, run_cli_command};

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
    let output = run_cli_command("device", &["list", "-o", "json"]);

    assert_success(&output, "device list -o json");
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
