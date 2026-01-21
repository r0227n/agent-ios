//! Common test utilities for CLI feature integration tests.
//!
//! Provides helper functions for running CLI commands and validating outputs.

#![allow(dead_code)]

use std::process::{Command, Output};

/// Re-export common functions from idb tests
pub use crate::idb_common::{ensure_companion_running, get_available_udid, get_test_bundle_id};

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
    serde_json::from_str(&stdout).expect(&format!("Failed to parse JSON: {}", stdout))
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
