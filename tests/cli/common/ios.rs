//! iOS utility functions for CLI integration tests.

/// Get a booted iOS simulator UDID using simctl.
///
/// # Panics
/// Panics if no booted simulator is available.
pub fn get_available_udid() -> String {
    let devices = agent_mobile_platform_ios::simctl::list_simulators()
        .expect("Failed to list simulators - ensure Xcode is installed");

    for device in &devices {
        if device.state.as_deref() == Some("Booted") {
            return device.udid.clone();
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

/// Ensure the device is ready for testing (no-op, kept for test compatibility).
pub fn ensure_companion_running(_udid: &str) {
    // No-op: simctl and XCUITest Runner are used directly.
}
