//! iOS utility functions for CLI integration tests.

use std::process::Command;

/// Get a booted iOS simulator UDID using simctl.
///
/// # Panics
/// Panics if no booted simulator is available.
pub fn get_available_udid() -> String {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "-j"])
        .output()
        .expect("Failed to execute xcrun simctl - ensure Xcode is installed");

    assert!(
        output.status.success(),
        "xcrun simctl list devices failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse simctl output as JSON");

    // Find a booted device
    if let Some(devices) = json.get("devices").and_then(|d| d.as_object()) {
        for (_runtime, device_list) in devices {
            if let Some(devices_array) = device_list.as_array() {
                for device in devices_array {
                    if device.get("state").and_then(|v| v.as_str()) == Some("Booted") {
                        if let Some(udid) = device.get("udid").and_then(|v| v.as_str()) {
                            return udid.to_string();
                        }
                    }
                }
            }
        }
    }

    panic!("No booted iOS simulator available. Start a simulator first.");
}

/// Get a test bundle ID for iOS.
///
/// Returns the bundle ID of a known test app, or a default.
pub fn get_test_bundle_id() -> String {
    // Safari is always available on iOS simulators
    "com.apple.mobilesafari".to_string()
}

/// Ensure the device is ready for testing.
///
/// Previously started idb_companion, now a no-op since we use
/// simctl and XCUITest Runner directly.
pub fn ensure_companion_running(_udid: &str) {
    // No-op: idb_companion is no longer needed.
}
