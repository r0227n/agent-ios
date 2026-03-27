//! iOS utility functions for CLI integration tests.

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

/// Get a booted iOS simulator UDID using simctl.
///
/// If no simulator is currently booted, boot the first available iPhone
/// simulator and wait until it is fully ready.
///
/// # Panics
/// Panics if no suitable simulator is available or if booting fails.
pub fn get_available_udid() -> String {
    let devices = agent_mobile_platform_ios::simctl::list_simulators()
        .expect("Failed to list simulators - ensure Xcode is installed");

    for device in &devices {
        if device.state.as_deref() == Some("Booted") {
            return device.udid.clone();
        }
    }

    let fallback = devices
        .iter()
        .find(|device| device.name.starts_with("iPhone"))
        .or_else(|| devices.first())
        .unwrap_or_else(|| {
            panic!("No iOS simulator available. Ensure Xcode simulators are installed.")
        });

    agent_mobile_platform_ios::simctl::boot(&fallback.udid)
        .unwrap_or_else(|err| panic!("Failed to boot simulator {}: {}", fallback.udid, err));

    let boot_status = Command::new("xcrun")
        .args(["simctl", "bootstatus", &fallback.udid, "-b"])
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "Failed to run simctl bootstatus for {}: {}",
                fallback.udid, err
            )
        });

    if !boot_status.status.success() {
        panic!(
            "Simulator {} failed to reach booted state:\nstdout: {}\nstderr: {}",
            fallback.udid,
            String::from_utf8_lossy(&boot_status.stdout),
            String::from_utf8_lossy(&boot_status.stderr)
        );
    }

    fallback.udid.clone()
}

/// Get a test bundle ID for iOS.
///
/// Returns the bundle ID of a known test app, or a default.
pub fn get_test_bundle_id() -> String {
    // The iOS CLI integration tests assert against Settings-specific text such as
    // "General", so use Settings as the canonical built-in app under test.
    "com.apple.Preferences".to_string()
}

/// Ensure the device is ready for testing (no-op, kept for test compatibility).
pub fn ensure_companion_running(_udid: &str) {
    // No-op: simctl and XCUITest Runner are used directly.
}

/// Stop the XCUITest Runner so tests can verify cold-start behavior.
pub fn stop_xcuitest_runner(udid: &str) {
    for bundle_id in [
        "com.agent-mobile.xcuitest-runner-uitests.xctrunner",
        "com.agent-mobile.xcuitest-runner",
    ] {
        let _ = Command::new("xcrun")
            .args(["simctl", "terminate", udid, bundle_id])
            .output();
    }

    let _ = Command::new("pkill")
        .args(["-f", "XCUITestRunner.xcodeproj.*testStartAutomationServer"])
        .output();

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let still_running = Command::new("pgrep")
            .args(["-f", "XCUITestRunner.xcodeproj.*testStartAutomationServer"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !still_running {
            return;
        }
        thread::sleep(Duration::from_millis(200));
    }
}
