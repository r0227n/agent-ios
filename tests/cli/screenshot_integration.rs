//! Screenshot feature integration tests.
//!
//! Tests for `agent-mobile screenshot` command.
//! Tests screen capture functionality.

use crate::common::{
    assert_failure, assert_success, assert_valid_jpeg_file, assert_valid_png_file,
    cleanup_temp_file, ensure_companion_running, get_available_udid, get_temp_file_path,
    run_cli_command_with_udid,
};

// ============================================================================
// Basic screenshot - Normal cases
// ============================================================================

/// Test basic screenshot command.
#[test]
fn test_screenshot_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("png");
    let output = run_cli_command_with_udid("screenshot", &["-o", &file_path], &udid);

    assert_success(&output, "screenshot basic");
    assert_valid_png_file(&file_path);

    cleanup_temp_file(&file_path);
}

/// Test screenshot with default output (current directory).
#[test]
fn test_screenshot_default_output() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // This will create a file in current directory
    let output = run_cli_command_with_udid("screenshot", &["-o", "/tmp/"], &udid);

    assert_success(&output, "screenshot to directory");

    // Find and clean up the created file
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("/tmp/") {
        // Try to extract path from output and clean up
        for line in stdout.lines() {
            if line.contains("/tmp/") && line.ends_with(".png") {
                let path = line.trim();
                if std::path::Path::new(path).exists() {
                    cleanup_temp_file(path);
                }
            }
        }
    }
}

// ============================================================================
// Output options - Normal cases
// ============================================================================

/// Test screenshot with specified file output.
#[test]
fn test_screenshot_file_output() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("png");
    let output = run_cli_command_with_udid("screenshot", &["-o", &file_path], &udid);

    assert_success(&output, "screenshot -o file");
    assert!(
        std::path::Path::new(&file_path).exists(),
        "Screenshot file should exist"
    );
    assert_valid_png_file(&file_path);

    cleanup_temp_file(&file_path);
}

/// Test screenshot with directory output.
#[test]
fn test_screenshot_directory_output() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("screenshot", &["-o", "/tmp/"], &udid);

    assert_success(&output, "screenshot -o /tmp/");

    // Output should mention the created file path
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("/tmp/") || stdout.contains("screenshot"),
        "Expected file path in output"
    );
}

// ============================================================================
// Format options - Normal cases
// ============================================================================

/// Test screenshot with PNG format (default).
#[test]
fn test_screenshot_png_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("png");
    let output = run_cli_command_with_udid("screenshot", &["-o", &file_path, "-f", "png"], &udid);

    assert_success(&output, "screenshot -f png");
    assert_valid_png_file(&file_path);

    cleanup_temp_file(&file_path);
}

/// Test screenshot with JPEG format.
#[test]
fn test_screenshot_jpeg_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("jpg");
    let output = run_cli_command_with_udid("screenshot", &["-o", &file_path, "-f", "jpeg"], &udid);

    assert_success(&output, "screenshot -f jpeg");
    assert_valid_jpeg_file(&file_path);

    cleanup_temp_file(&file_path);
}

/// Test screenshot format from file extension.
#[test]
fn test_screenshot_format_from_extension() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // PNG by extension
    let png_path = get_temp_file_path("png");
    let output1 = run_cli_command_with_udid("screenshot", &["-o", &png_path], &udid);
    assert_success(&output1, "screenshot .png extension");
    assert_valid_png_file(&png_path);
    cleanup_temp_file(&png_path);
}

// ============================================================================
// File creation - Normal cases
// ============================================================================

/// Test screenshot creates non-empty file.
#[test]
fn test_screenshot_file_not_empty() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("png");
    let output = run_cli_command_with_udid("screenshot", &["-o", &file_path], &udid);

    assert_success(&output, "screenshot creates file");

    let metadata = std::fs::metadata(&file_path).expect("Failed to get file metadata");
    assert!(metadata.len() > 0, "Screenshot file should not be empty");

    cleanup_temp_file(&file_path);
}

/// Test screenshot overwrites existing file.
#[test]
fn test_screenshot_overwrites_existing() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("png");

    // Create initial file
    std::fs::write(&file_path, "dummy").expect("Failed to create dummy file");

    // Screenshot should overwrite
    let output = run_cli_command_with_udid("screenshot", &["-o", &file_path], &udid);

    assert_success(&output, "screenshot overwrites");

    // Should now be a valid PNG
    assert_valid_png_file(&file_path);

    cleanup_temp_file(&file_path);
}

// ============================================================================
// Error cases
// ============================================================================

/// Test screenshot with invalid UDID.
#[test]
fn test_screenshot_invalid_udid() {
    let file_path = get_temp_file_path("png");
    let output = run_cli_command_with_udid("screenshot", &["-o", &file_path], "invalid-udid-12345");

    assert_failure(&output, "screenshot with invalid UDID");

    // File should not exist or be empty
    if std::path::Path::new(&file_path).exists() {
        cleanup_temp_file(&file_path);
    }
}

/// Test screenshot with invalid format.
#[test]
fn test_screenshot_invalid_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("png");
    let output =
        run_cli_command_with_udid("screenshot", &["-o", &file_path, "-f", "invalid"], &udid);

    assert_failure(&output, "screenshot -f invalid");

    cleanup_temp_file(&file_path);
}

/// Test screenshot with permission denied path.
#[test]
fn test_screenshot_permission_denied() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Try to write to a protected location
    let output = run_cli_command_with_udid("screenshot", &["-o", "/etc/screenshot.png"], &udid);

    assert_failure(&output, "screenshot to /etc/");
}

/// Test screenshot with non-existent directory.
#[test]
fn test_screenshot_non_existent_directory() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid(
        "screenshot",
        &["-o", "/nonexistent/directory/screenshot.png"],
        &udid,
    );

    assert_failure(&output, "screenshot to non-existent directory");
}

// ============================================================================
// Multiple screenshots
// ============================================================================

/// Test taking multiple screenshots.
#[test]
fn test_screenshot_multiple() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file1 = get_temp_file_path("png");
    let file2 = get_temp_file_path("png");

    let output1 = run_cli_command_with_udid("screenshot", &["-o", &file1], &udid);
    let output2 = run_cli_command_with_udid("screenshot", &["-o", &file2], &udid);

    assert_success(&output1, "screenshot 1");
    assert_success(&output2, "screenshot 2");

    assert_valid_png_file(&file1);
    assert_valid_png_file(&file2);

    cleanup_temp_file(&file1);
    cleanup_temp_file(&file2);
}

// ============================================================================
// Platform-agnostic screenshot tests
// ============================================================================

/// Test screenshot works on any platform.
#[test]
fn test_screenshot_platform_agnostic() {
    use crate::common::{run_cli_command_with_device, DeviceIdentifier};

    let device = match DeviceIdentifier::get_any_available() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Skipping test: No device available");
            return;
        }
    };

    device.ensure_ready();

    let file_path = get_temp_file_path("png");
    let output = run_cli_command_with_device("screenshot", &["-o", &file_path], &device);

    assert_success(
        &output,
        &format!("screenshot on {}", device.platform_name()),
    );
    assert_valid_png_file(&file_path);

    cleanup_temp_file(&file_path);
}

/// Test screenshot format handling on any platform.
#[test]
fn test_screenshot_format_platform_specific() {
    use crate::common::{run_cli_command_with_device, DeviceIdentifier, TestPlatform};

    let device = match DeviceIdentifier::get_any_available() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Skipping test: No device available");
            return;
        }
    };

    device.ensure_ready();

    let file_path = get_temp_file_path("png");

    match device.platform {
        TestPlatform::Ios => {
            // iOS supports both PNG and JPEG
            let jpeg_path = get_temp_file_path("jpg");
            let output_jpeg = run_cli_command_with_device(
                "screenshot",
                &["-o", &jpeg_path, "-f", "jpeg"],
                &device,
            );
            assert_success(&output_jpeg, "iOS screenshot jpeg");
            assert_valid_jpeg_file(&jpeg_path);
            cleanup_temp_file(&jpeg_path);
        }
        TestPlatform::Android => {
            // Android primarily supports PNG
            let output_png =
                run_cli_command_with_device("screenshot", &["-o", &file_path], &device);
            assert_success(&output_png, "Android screenshot png");
            assert_valid_png_file(&file_path);
        }
    }

    cleanup_temp_file(&file_path);
}
