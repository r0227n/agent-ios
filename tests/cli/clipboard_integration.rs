//! Clipboard feature integration tests.
//!
//! Tests for `agent-mobile clipboard` commands.
//! Note: Clipboard is iOS-only via xcrun simctl pbcopy/pbpaste.
//!
//! IMPORTANT: These tests share the system clipboard and must run sequentially.
//! Use `cargo test --test cli clipboard -- --test-threads=1` for reliable results.

use crate::common::{assert_success, get_available_udid, get_stdout, run_cli_command_with_udid};

/// Test clipboard --copy command (basic copy operation).
#[test]
fn test_clipboard_copy() {
    let udid = get_available_udid();

    let output = run_cli_command_with_udid("clipboard", &["--copy", "TestCopyBasic"], &udid);

    assert_success(&output, "clipboard --copy TestCopyBasic");
}

/// Test clipboard --paste command (includes copy for isolation).
#[test]
fn test_clipboard_paste() {
    let udid = get_available_udid();

    // Use a unique value for this test
    let unique_text = "PasteTest_unique_42";

    // Copy first to ensure clean state
    let copy_output = run_cli_command_with_udid("clipboard", &["--copy", unique_text], &udid);
    assert_success(&copy_output, "clipboard --copy (setup)");

    // Then paste immediately
    let output = run_cli_command_with_udid("clipboard", &["--paste"], &udid);

    assert_success(&output, "clipboard --paste");
    let stdout = get_stdout(&output);
    assert!(
        stdout.contains(unique_text),
        "Expected '{}' in clipboard, got: {}",
        unique_text,
        stdout
    );
}

/// Test clipboard roundtrip (copy -> paste) with unique value.
#[test]
fn test_clipboard_roundtrip() {
    let udid = get_available_udid();

    // Use a unique value with timestamp-like suffix
    let test_text = "RoundtripTest_abc_789";

    // Copy
    let copy_output = run_cli_command_with_udid("clipboard", &["--copy", test_text], &udid);
    assert_success(&copy_output, "clipboard --copy");

    // Paste immediately
    let paste_output = run_cli_command_with_udid("clipboard", &["--paste"], &udid);
    assert_success(&paste_output, "clipboard --paste");

    let stdout = get_stdout(&paste_output);
    assert!(
        stdout.contains(test_text),
        "Clipboard roundtrip failed. Expected '{}', got: {}",
        test_text,
        stdout
    );
}

/// Test clipboard --copy with special characters.
#[test]
fn test_clipboard_special_chars() {
    let udid = get_available_udid();

    // Use unique special chars string
    let test_text = "SpecialChars_!@#$_xyz";

    // Copy with special characters
    let copy_output = run_cli_command_with_udid("clipboard", &["--copy", test_text], &udid);
    assert_success(&copy_output, "clipboard --copy (special chars)");

    // Paste immediately
    let paste_output = run_cli_command_with_udid("clipboard", &["--paste"], &udid);
    assert_success(&paste_output, "clipboard --paste");

    let stdout = get_stdout(&paste_output);
    assert!(
        stdout.contains(test_text),
        "Expected '{}' in clipboard, got: {}",
        test_text,
        stdout
    );
}
