//! {CommandName} integration tests
//!
//! Tests for: agent-mobile {COMMAND_NAME}

use std::process::Output;

#[path = "common/mod.rs"]
mod common;

/// Run {COMMAND_NAME} command
fn run_command(args: &[&str]) -> Output {
    common::run_cli_command("{COMMAND_NAME}", args)
}

/// Run {COMMAND_NAME} command with UDID
fn run_command_with_udid(args: &[&str], udid: &str) -> Output {
    common::run_cli_command_with_udid("{COMMAND_NAME}", args, udid)
}

/// Test {COMMAND_NAME} command success case
#[test]
fn test_{COMMAND_NAME}_success() {
    let udid = common::get_available_udid();
    common::ensure_device_ready(&udid);

    // TODO: Customize test arguments
    let output = run_command_with_udid(&[], &udid);

    common::assert_success(&output, "{CommandName} command");

    // TODO: Add assertions for output content
    // Example:
    // let stdout = String::from_utf8_lossy(&output.stdout);
    // assert!(stdout.contains("expected text"));
}

/// Test {COMMAND_NAME} with invalid device
#[test]
fn test_{COMMAND_NAME}_invalid_device() {
    let output = run_command_with_udid(&[], "invalid-udid-12345");

    // Expect failure with invalid UDID
    assert!(
        !output.status.success(),
        "Should fail with invalid UDID"
    );

    // Verify error message is actionable
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("device") || stderr.contains("UDID") || stderr.contains("not found"),
        "Error message should mention device/UDID: {}",
        stderr
    );
}

// TODO: Add more test cases
// - Test with different argument combinations
// - Test edge cases
// - Test error handling
//
// Example:
// #[test]
// fn test_{COMMAND_NAME}_custom_arg() {
//     let udid = common::get_available_udid();
//     common::ensure_device_ready(&udid);
//
//     let output = run_command_with_udid(&["--custom-arg", "value"], &udid);
//     common::assert_success(&output, "{CommandName} with custom arg");
// }
