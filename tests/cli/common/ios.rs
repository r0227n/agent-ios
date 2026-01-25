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

/// Ensure the idb companion is running for the given device.
///
/// This starts the companion if not already running.
pub fn ensure_companion_running(udid: &str) {
    // Check if companion is already running by trying to connect
    let socket_path = format!(
        "{}/Library/Developer/idb/idb_companion_{}.sock",
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        udid
    );

    if std::path::Path::new(&socket_path).exists() {
        return; // Companion already running
    }

    // Start companion
    let output = Command::new("idb_companion").args(["--udid", udid]).spawn();

    if output.is_err() {
        // Companion might not be installed, which is OK for simctl-based tests
        eprintln!("Note: idb_companion not available, tests will use simctl directly");
    }

    // Give companion time to start
    std::thread::sleep(std::time::Duration::from_millis(500));
}
