mod common;

use common::{get_available_udid, get_test_app_path};
use std::process::Command;

#[test]
fn test_install_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "install", "--help"])
        .output()
        .expect("Failed to run install command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bundle_path") || stdout.contains("BUNDLE_PATH"));
    assert!(stdout.contains("--make-debuggable"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("--compression"));
    assert!(stdout.contains("--override-mtime"));
}

#[test]
fn test_install_with_udid() {
    let udid = get_available_udid();
    let app_path = get_test_app_path();

    // Skip if test app doesn't exist
    if !std::path::Path::new(&app_path).exists() {
        eprintln!(
            "Skipping test_install_with_udid: test app not found at {}",
            app_path
        );
        eprintln!("Set TEST_APP_PATH environment variable to specify a different path");
        return;
    }

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "install", "--udid", &udid, &app_path])
        .output()
        .expect("Failed to run install command");

    assert!(
        output.status.success(),
        "Install failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("Installed: "),
        "Expected output to start with 'Installed: ', got: {}",
        stdout
    );
}

#[test]
fn test_install_json_output() {
    let udid = get_available_udid();
    let app_path = get_test_app_path();

    // Skip if test app doesn't exist
    if !std::path::Path::new(&app_path).exists() {
        eprintln!(
            "Skipping test_install_json_output: test app not found at {}",
            app_path
        );
        eprintln!("Set TEST_APP_PATH environment variable to specify a different path");
        return;
    }

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "install", "--json", "--udid", &udid, &app_path])
        .output()
        .expect("Failed to run install command");

    assert!(
        output.status.success(),
        "Install failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(
        json.get("installedAppBundleId").is_some(),
        "JSON should contain installedAppBundleId field"
    );
    assert!(json.get("uuid").is_some(), "JSON should contain uuid field");
}

#[test]
fn test_install_invalid_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "install",
            "--udid",
            "INVALID_UDID_12345",
            "/tmp/test.app",
        ])
        .output()
        .expect("Failed to run install command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No companion found for UDID"),
        "Expected 'No companion found for UDID' error, got: {}",
        stderr
    );
}

#[test]
fn test_install_without_udid() {
    let app_path = get_test_app_path();

    // Skip if test app doesn't exist
    if !std::path::Path::new(&app_path).exists() {
        eprintln!(
            "Skipping test_install_without_udid: test app not found at {}",
            app_path
        );
        eprintln!("Set TEST_APP_PATH environment variable to specify a different path");
        return;
    }

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "install", &app_path])
        .output()
        .expect("Failed to run install command");

    // Should succeed if there's at least one companion available
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.starts_with("Installed: "),
            "Expected output to start with 'Installed: ', got: {}",
            stdout
        );
    } else {
        // Or fail with "No companions available" if no companion is running
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available") || stderr.contains("No companion found"),
            "Unexpected error: {}",
            stderr
        );
    }
}

#[test]
fn test_install_compatibility_with_python_idb() {
    let udid = get_available_udid();
    let app_path = get_test_app_path();

    // Skip if test app doesn't exist
    if !std::path::Path::new(&app_path).exists() {
        eprintln!(
            "Skipping test_install_compatibility_with_python_idb: test app not found at {}",
            app_path
        );
        eprintln!("Set TEST_APP_PATH environment variable to specify a different path");
        return;
    }

    // Python idb
    let python_output = Command::new("idb")
        .args(["install", "--udid", &udid, &app_path])
        .output()
        .expect("Failed to run Python idb");

    // Rust implementation
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "install", "--udid", &udid, &app_path])
        .output()
        .expect("Failed to run Rust implementation");

    // Both should succeed
    assert_eq!(
        rust_output.status.success(),
        python_output.status.success(),
        "Exit codes differ: Rust={}, Python={}",
        rust_output.status,
        python_output.status
    );

    if rust_output.status.success() {
        // Compare output format (both start with "Installed: ")
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);

        assert!(
            rust_stdout.starts_with("Installed: "),
            "Rust output should start with 'Installed: ', got: {}",
            rust_stdout
        );
        assert!(
            python_stdout.starts_with("Installed: "),
            "Python output should start with 'Installed: ', got: {}",
            python_stdout
        );

        // Extract bundle ID (first word after "Installed: ")
        let rust_bundle_id = rust_stdout.split_whitespace().nth(1);
        let python_bundle_id = python_stdout.split_whitespace().nth(1);

        assert_eq!(
            rust_bundle_id, python_bundle_id,
            "Bundle IDs should match: Rust={:?}, Python={:?}",
            rust_bundle_id, python_bundle_id
        );
    }
}

#[test]
fn test_install_nonexistent_app() {
    let udid = get_available_udid();

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "install",
            "--udid",
            &udid,
            "/nonexistent/path/to/app.app",
        ])
        .output()
        .expect("Failed to run install command");

    assert!(!output.status.success(), "Should fail for nonexistent app");
}
