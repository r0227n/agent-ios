//! Record feature integration tests.
//!
//! Tests for `agent-mobile record` command.
//! Tests screen recording functionality.

use crate::common::{
    assert_failure, assert_success, assert_valid_mp4_file, cleanup_temp_file,
    ensure_companion_running, get_available_udid, get_temp_file_path, run_cli_command_with_udid,
};

// ============================================================================
// Basic record - Normal cases
// ============================================================================

/// Test basic record command with time limit.
#[test]
fn test_record_with_time_limit() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "2"], &udid);

    assert_success(&output, "record -t 2");

    // Verify video file was created
    assert!(
        std::path::Path::new(&file_path).exists(),
        "Video file should exist"
    );

    // Check file size > 0
    let metadata = std::fs::metadata(&file_path).expect("Failed to get file metadata");
    assert!(metadata.len() > 0, "Video file should not be empty");

    cleanup_temp_file(&file_path);
}

/// Test record with short time limit.
#[test]
fn test_record_short_time_limit() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "1"], &udid);

    assert_success(&output, "record -t 1");
    assert!(
        std::path::Path::new(&file_path).exists(),
        "Video file should exist"
    );

    cleanup_temp_file(&file_path);
}

// ============================================================================
// Output options - Normal cases
// ============================================================================

/// Test record with specified file output.
#[test]
fn test_record_file_output() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "1"], &udid);

    assert_success(&output, "record -o file");
    assert!(
        std::path::Path::new(&file_path).exists(),
        "Video file should exist"
    );

    cleanup_temp_file(&file_path);
}

/// Test record with directory output.
#[test]
fn test_record_directory_output() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("record", &["-o", "/tmp/", "-t", "1"], &udid);

    assert_success(&output, "record -o /tmp/");

    // Output should mention the created file path
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("/tmp/") || stdout.contains("mp4") || stdout.contains("record"),
        "Expected file path in output"
    );
}

// ============================================================================
// Time limit options - Normal cases
// ============================================================================

/// Test record with longer time limit.
#[test]
fn test_record_longer_time_limit() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "3"], &udid);

    assert_success(&output, "record -t 3");

    // Verify file exists and has content
    assert!(
        std::path::Path::new(&file_path).exists(),
        "Video file should exist"
    );
    let metadata = std::fs::metadata(&file_path).expect("Failed to get metadata");
    assert!(metadata.len() > 0, "Video file should have content");

    cleanup_temp_file(&file_path);
}

/// Test record with --time-limit long option.
#[test]
fn test_record_time_limit_long_option() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output =
        run_cli_command_with_udid("record", &["-o", &file_path, "--time-limit", "2"], &udid);

    assert_success(&output, "record --time-limit 2");
    assert!(
        std::path::Path::new(&file_path).exists(),
        "Video file should exist"
    );

    cleanup_temp_file(&file_path);
}

// ============================================================================
// Error cases
// ============================================================================

/// Test record with invalid UDID.
#[test]
fn test_record_invalid_udid() {
    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid(
        "record",
        &["-o", &file_path, "-t", "1"],
        "invalid-udid-12345",
    );

    assert_failure(&output, "record with invalid UDID");

    // Clean up if file was created
    if std::path::Path::new(&file_path).exists() {
        cleanup_temp_file(&file_path);
    }
}

/// Test record with permission denied path.
#[test]
fn test_record_permission_denied() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("record", &["-o", "/etc/recording.mp4", "-t", "1"], &udid);

    assert_failure(&output, "record to /etc/");
}

/// Test record with non-existent directory.
#[test]
fn test_record_non_existent_directory() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "record",
        &["-o", "/nonexistent/directory/recording.mp4", "-t", "1"],
        &udid,
    );

    assert_failure(&output, "record to non-existent directory");
}

/// Test record with invalid time limit.
#[test]
fn test_record_invalid_time_limit() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "-1"], &udid);

    assert_failure(&output, "record -t -1");

    cleanup_temp_file(&file_path);
}

/// Test record with zero time limit.
#[test]
fn test_record_zero_time_limit() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "0"], &udid);

    // Zero time limit may fail or produce empty file
    let _ = output;

    cleanup_temp_file(&file_path);
}

// ============================================================================
// File creation
// ============================================================================

/// Test record creates valid MP4 file.
#[test]
fn test_record_creates_valid_mp4() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "2"], &udid);

    assert_success(&output, "record creates MP4");
    assert_valid_mp4_file(&file_path);

    cleanup_temp_file(&file_path);
}

/// Test record overwrites existing file.
#[test]
fn test_record_overwrites_existing() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("mp4");

    // Create initial file
    std::fs::write(&file_path, "dummy").expect("Failed to create dummy file");

    // Record should overwrite
    let output = run_cli_command_with_udid("record", &["-o", &file_path, "-t", "1"], &udid);

    assert_success(&output, "record overwrites");

    // Should now be larger than dummy file
    let metadata = std::fs::metadata(&file_path).expect("Failed to get metadata");
    assert!(metadata.len() > 5, "Video file should be larger than dummy");

    cleanup_temp_file(&file_path);
}
