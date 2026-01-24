//! Common test utilities for CLI feature integration tests.
//!
//! Provides helper functions for running CLI commands and validating outputs.

#![allow(dead_code)]

use std::process::{Command, Output};

// Platform abstraction modules
mod android;
mod platform;

// Re-export iOS common functions from idb tests
pub use crate::idb_common::{ensure_companion_running, get_available_udid, get_test_bundle_id};

// Re-export Android utilities
pub use android::get_available_serial;

// Re-export platform abstraction
pub use platform::{DeviceIdentifier, TestPlatform};

// Re-export for potential future use
#[allow(unused_imports)]
pub use platform::detect_platform;

/// Run an agent-mobile CLI feature command.
///
/// Example: run_cli_command("app", &["--list", "--udid", "ABC123"])
pub fn run_cli_command(feature: &str, args: &[&str]) -> Output {
    let mut full_args = vec![feature];
    full_args.extend_from_slice(args);

    Command::new("./target/debug/agent-mobile")
        .args(&full_args)
        .output()
        .expect("Failed to run agent-mobile - ensure it is built with 'cargo build'")
}

/// Run an agent-mobile CLI feature command with UDID.
///
/// Automatically appends --udid <udid> to the arguments.
pub fn run_cli_command_with_udid(feature: &str, args: &[&str], udid: &str) -> Output {
    let mut full_args: Vec<&str> = args.to_vec();
    full_args.push("--udid");
    full_args.push(udid);

    run_cli_command(feature, &full_args)
}

/// Run an agent-mobile CLI feature command with DeviceIdentifier.
///
/// This is the recommended way to write platform-agnostic tests.
/// The device platform is automatically detected from the device ID.
///
/// # Example
/// ```
/// let device = DeviceIdentifier::get_any_available().unwrap();
/// let output = run_cli_command_with_device("app", &["list"], &device);
/// ```
pub fn run_cli_command_with_device(
    feature: &str,
    args: &[&str],
    device: &DeviceIdentifier,
) -> Output {
    run_cli_command_with_udid(feature, args, &device.id)
}

/// Assert that the command succeeded.
pub fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{} failed with exit code {:?}:\nstdout: {}\nstderr: {}",
        context,
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Assert that the command failed.
pub fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{} should have failed but succeeded:\nstdout: {}\nstderr: {}",
        context,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Assert that the output is valid JSON and return parsed value.
pub fn assert_valid_json(output: &Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("Failed to parse JSON: {}", stdout))
}

/// Assert that the output contains expected text.
pub fn assert_stdout_contains(output: &Output, expected: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(expected),
        "Expected stdout to contain '{}', but got: {}",
        expected,
        stdout
    );
}

/// Assert that stderr contains expected text.
pub fn assert_stderr_contains(output: &Output, expected: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "Expected stderr to contain '{}', but got: {}",
        expected,
        stderr
    );
}

/// Get stdout as string.
pub fn get_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Get stderr as string.
pub fn get_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Assert that exit code is 0.
pub fn assert_exit_code_0(output: &Output, context: &str) {
    assert!(
        output.status.code() == Some(0),
        "{} should have exit code 0, but got {:?}:\nstdout: {}\nstderr: {}",
        context,
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Assert that exit code is 1.
pub fn assert_exit_code_1(output: &Output, context: &str) {
    assert!(
        output.status.code() == Some(1),
        "{} should have exit code 1, but got {:?}:\nstdout: {}\nstderr: {}",
        context,
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Generate a temporary file path with the given extension.
pub fn get_temp_file_path(extension: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("/tmp/agent-mobile-test-{}.{}", timestamp, extension)
}

/// Assert that a file exists and is a valid PNG file (checks magic bytes).
pub fn assert_valid_png_file(path: &str) {
    use std::fs;
    let data = fs::read(path).unwrap_or_else(|_| panic!("Failed to read file: {}", path));
    assert!(data.len() >= 8, "PNG file too small: {}", path);
    // PNG magic bytes: 89 50 4E 47 0D 0A 1A 0A
    assert_eq!(
        &data[0..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        "Invalid PNG magic bytes in: {}",
        path
    );
}

/// Assert that a file exists and is a valid JPEG file (checks magic bytes).
pub fn assert_valid_jpeg_file(path: &str) {
    use std::fs;
    let data = fs::read(path).unwrap_or_else(|_| panic!("Failed to read file: {}", path));
    assert!(data.len() >= 2, "JPEG file too small: {}", path);
    // JPEG magic bytes: FF D8
    assert_eq!(
        &data[0..2],
        &[0xFF, 0xD8],
        "Invalid JPEG magic bytes in: {}",
        path
    );
}

/// Assert that a file exists and is a valid MP4 file (checks for ftyp box).
pub fn assert_valid_mp4_file(path: &str) {
    use std::fs;
    let data = fs::read(path).unwrap_or_else(|_| panic!("Failed to read file: {}", path));
    assert!(data.len() >= 12, "MP4 file too small: {}", path);
    // MP4 files have "ftyp" at bytes 4-7
    let ftyp = String::from_utf8_lossy(&data[4..8]);
    assert_eq!(
        ftyp, "ftyp",
        "Invalid MP4 format (missing ftyp box) in: {}",
        path
    );
}

/// Assert that JSON value has a specific field.
pub fn assert_json_has_field(json: &serde_json::Value, field: &str) {
    assert!(
        json.get(field).is_some(),
        "Expected JSON to have field '{}', got: {:?}",
        field,
        json
    );
}

/// Assert that JSON value is a non-empty array.
pub fn assert_json_array_not_empty(json: &serde_json::Value) {
    assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
    assert!(
        !json.as_array().unwrap().is_empty(),
        "Expected non-empty JSON array"
    );
}

/// Get first button reference from accessibility snapshot.
/// Returns element reference string (e.g., "$1").
pub fn get_first_button_ref(udid: &str) -> String {
    let output = run_cli_command_with_udid("snapshot", &["-f", "json"], udid);
    assert_success(&output, "snapshot for button ref");

    let json = assert_valid_json(&output);
    find_first_element_ref(&json, "Button").unwrap_or_else(|| {
        panic!("No Button element found in accessibility snapshot");
    })
}

/// Get first text field reference from accessibility snapshot.
pub fn get_first_text_field_ref(udid: &str) -> String {
    let output = run_cli_command_with_udid("snapshot", &["-f", "json"], udid);
    assert_success(&output, "snapshot for text field ref");

    let json = assert_valid_json(&output);
    find_first_element_ref(&json, "TextField").unwrap_or_else(|| {
        panic!("No TextField element found in accessibility snapshot");
    })
}

/// Recursively find first element with given type and return its reference.
fn find_first_element_ref(json: &serde_json::Value, element_type: &str) -> Option<String> {
    if let Some(obj) = json.as_object() {
        if let Some(etype) = obj.get("type").and_then(|v| v.as_str()) {
            if etype == element_type {
                if let Some(ref_val) = obj.get("ref").and_then(|v| v.as_str()) {
                    return Some(ref_val.to_string());
                }
            }
        }
        if let Some(children) = obj.get("children").and_then(|v| v.as_array()) {
            for child in children {
                if let Some(found) = find_first_element_ref(child, element_type) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// Clean up a temporary file if it exists.
pub fn cleanup_temp_file(path: &str) {
    let _ = std::fs::remove_file(path);
}
