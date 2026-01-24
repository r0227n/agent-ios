//! Snapshot feature integration tests.
//!
//! Tests for `agent-mobile snapshot` command.
//! Tests UI snapshot capture functionality.

use crate::common::{
    assert_failure, assert_success, assert_valid_json, cleanup_temp_file, ensure_companion_running,
    get_available_udid, get_temp_file_path, get_test_bundle_id, run_cli_command_with_udid,
};

// ============================================================================
// Basic snapshot - Normal cases
// ============================================================================

/// Test basic snapshot command.
#[test]
fn test_snapshot_basic() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &[], &udid);

    assert_success(&output, "snapshot basic");
}

/// Test snapshot with JSON format.
#[test]
fn test_snapshot_json_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-f", "json"], &udid);

    assert_success(&output, "snapshot -f json");
    let _ = assert_valid_json(&output);
}

/// Test snapshot with text format (default).
#[test]
fn test_snapshot_text_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-f", "text"], &udid);

    assert_success(&output, "snapshot -f text");
    // Should contain element references like @e1
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("@e") || stdout.contains("$"),
        "Expected element references in snapshot output"
    );
}

// ============================================================================
// Interactive option - Normal cases
// ============================================================================

/// Test snapshot with --interactive option.
#[test]
fn test_snapshot_interactive() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--interactive"], &udid);

    assert_success(&output, "snapshot --interactive");
}

/// Test snapshot with -i short option.
#[test]
fn test_snapshot_interactive_short() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-i"], &udid);

    assert_success(&output, "snapshot -i");
}

/// Test snapshot interactive with JSON format.
#[test]
fn test_snapshot_interactive_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-i", "-f", "json"], &udid);

    assert_success(&output, "snapshot -i -f json");
    let _ = assert_valid_json(&output);
}

// ============================================================================
// Compact option - Normal cases
// ============================================================================

/// Test snapshot with --compact option.
#[test]
fn test_snapshot_compact() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--compact"], &udid);

    assert_success(&output, "snapshot --compact");
}

/// Test snapshot with -c short option.
#[test]
fn test_snapshot_compact_short() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-c"], &udid);

    assert_success(&output, "snapshot -c");
}

// ============================================================================
// Depth option - Normal cases
// ============================================================================

/// Test snapshot with --depth option.
#[test]
fn test_snapshot_depth() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--depth", "3"], &udid);

    assert_success(&output, "snapshot --depth 3");
}

/// Test snapshot with -d short option.
#[test]
fn test_snapshot_depth_short() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-d", "2"], &udid);

    assert_success(&output, "snapshot -d 2");
}

/// Test snapshot with depth 1.
#[test]
fn test_snapshot_depth_one() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--depth", "1"], &udid);

    assert_success(&output, "snapshot --depth 1");
}

// ============================================================================
// Scope option - Normal cases
// ============================================================================

/// Test snapshot with --scope using text.
#[test]
fn test_snapshot_scope_text() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--scope", "General"], &udid);

    assert_success(&output, "snapshot --scope General");
}

/// Test snapshot with -s short option.
#[test]
fn test_snapshot_scope_short() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-s", "General"], &udid);

    assert_success(&output, "snapshot -s General");
}

// ============================================================================
// Output file option - Normal cases
// ============================================================================

/// Test snapshot with --output to file.
#[test]
fn test_snapshot_output_file() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let file_path = get_temp_file_path("txt");
    let output = run_cli_command_with_udid("snapshot", &["-o", &file_path], &udid);

    assert_success(&output, "snapshot -o file");

    // Verify file was created
    assert!(
        std::path::Path::new(&file_path).exists(),
        "Output file should exist"
    );

    cleanup_temp_file(&file_path);
}

/// Test snapshot with --output JSON file.
#[test]
fn test_snapshot_output_json_file() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let file_path = get_temp_file_path("json");
    let output = run_cli_command_with_udid("snapshot", &["-o", &file_path, "-f", "json"], &udid);

    assert_success(&output, "snapshot -o file -f json");

    // Verify file was created and is valid JSON
    let content = std::fs::read_to_string(&file_path).expect("Failed to read output file");
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("Output file is not valid JSON");

    cleanup_temp_file(&file_path);
}

// ============================================================================
// Scroll options - Normal cases
// ============================================================================

/// Test snapshot with --no-scroll option.
#[test]
fn test_snapshot_no_scroll() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--no-scroll"], &udid);

    assert_success(&output, "snapshot --no-scroll");
}

/// Test snapshot with --max-scrolls option.
#[test]
fn test_snapshot_max_scrolls() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--max-scrolls", "2"], &udid);

    assert_success(&output, "snapshot --max-scrolls 2");
}

/// Test snapshot with --scroll-delay option.
#[test]
fn test_snapshot_scroll_delay() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["--scroll-delay", "200"], &udid);

    assert_success(&output, "snapshot --scroll-delay 200");
}

// ============================================================================
// Option combinations - Normal cases
// ============================================================================

/// Test snapshot with multiple options combined.
#[test]
fn test_snapshot_combined_options() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output =
        run_cli_command_with_udid("snapshot", &["-i", "-c", "-d", "3", "-f", "json"], &udid);

    assert_success(&output, "snapshot -i -c -d 3 -f json");
    let _ = assert_valid_json(&output);
}

/// Test snapshot with interactive and compact.
#[test]
fn test_snapshot_interactive_compact() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let _ = run_cli_command_with_udid("app", &["launch", &get_test_bundle_id()], &udid);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_udid("snapshot", &["-i", "-c"], &udid);

    assert_success(&output, "snapshot -i -c");
}

// ============================================================================
// Error cases
// ============================================================================

/// Test snapshot with invalid depth value.
#[test]
fn test_snapshot_invalid_depth() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("snapshot", &["--depth", "-1"], &udid);

    assert_failure(&output, "snapshot --depth -1");
}

/// Test snapshot with invalid scope reference.
#[test]
fn test_snapshot_invalid_scope() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output =
        run_cli_command_with_udid("snapshot", &["--scope", "NonExistentElement12345"], &udid);

    assert_failure(&output, "snapshot --scope invalid");
}

/// Test snapshot with invalid UDID.
#[test]
fn test_snapshot_invalid_udid() {
    let output = run_cli_command_with_udid("snapshot", &[], "invalid-udid-12345");

    assert_failure(&output, "snapshot with invalid UDID");
}

/// Test snapshot with invalid format.
#[test]
fn test_snapshot_invalid_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = run_cli_command_with_udid("snapshot", &["-f", "invalid"], &udid);

    assert_failure(&output, "snapshot -f invalid");
}

// ============================================================================
// Platform-agnostic snapshot tests
// ============================================================================

/// Test snapshot works on any platform.
#[test]
fn test_snapshot_platform_agnostic() {
    use crate::common::{run_cli_command_with_device, DeviceIdentifier};

    let device = match DeviceIdentifier::get_any_available() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Skipping test: No device available");
            return;
        }
    };

    device.ensure_ready();

    // Launch test app
    let bundle_id = device.get_test_bundle_id();
    let _ = run_cli_command_with_device("app", &["launch", &bundle_id], &device);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_device("snapshot", &[], &device);
    assert_success(&output, &format!("snapshot on {}", device.platform_name()));
}

/// Test snapshot JSON format on any platform.
#[test]
fn test_snapshot_json_platform_agnostic() {
    use crate::common::{run_cli_command_with_device, DeviceIdentifier};

    let device = match DeviceIdentifier::get_any_available() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Skipping test: No device available");
            return;
        }
    };

    device.ensure_ready();

    let bundle_id = device.get_test_bundle_id();
    let _ = run_cli_command_with_device("app", &["launch", &bundle_id], &device);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = run_cli_command_with_device("snapshot", &["-f", "json"], &device);
    assert_success(
        &output,
        &format!("snapshot json on {}", device.platform_name()),
    );
    let _ = assert_valid_json(&output);
}

// ============================================================================
// Android-specific snapshot tests
// ============================================================================

/// Test snapshot contains Android-specific element types.
#[test]
fn test_snapshot_android_element_types() {
    use crate::common::{get_available_serial, run_cli_command_with_udid};

    // Skip if no Android device available
    let serial = match std::panic::catch_unwind(|| get_available_serial()) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: No Android device available");
            return;
        }
    };

    // Launch Settings app
    let package = "com.android.settings";
    let _ = run_cli_command_with_udid("app", &["launch", package], &serial);
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let output = run_cli_command_with_udid("snapshot", &["-f", "json"], &serial);
    assert_success(&output, "snapshot on Android");

    let json = assert_valid_json(&output);

    // Android-specific element types should be present
    let json_str = serde_json::to_string(&json).unwrap();
    let has_android_types = json_str.contains("TextView")
        || json_str.contains("Button")
        || json_str.contains("EditText")
        || json_str.contains("FrameLayout");

    assert!(
        has_android_types,
        "Expected Android-specific element types in snapshot"
    );
}
