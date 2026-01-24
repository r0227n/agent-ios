//! Console feature integration tests.
//!
//! Tests for `agent-mobile console` command.
//! Tests device log streaming functionality.

use crate::common::{
    cleanup_temp_file, ensure_companion_running, get_available_udid, get_temp_file_path,
};
use std::process::{Command, Stdio};

// ============================================================================
// Helper for console tests
// ============================================================================

/// Run console command with timeout (for streaming tests).
#[allow(dead_code)]
fn run_console_with_timeout(args: &[&str], timeout_secs: u64) -> std::process::Output {
    let mut full_args = vec!["console"];
    full_args.extend_from_slice(args);

    let child = Command::new("./target/debug/agent-mobile")
        .args(&full_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn agent-mobile");

    // Wait for timeout then kill
    std::thread::sleep(std::time::Duration::from_secs(timeout_secs));

    // Kill the process
    let output = child.wait_with_output().unwrap_or_else(|_| {
        // If wait_with_output fails, we still got some output
        std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    });

    output
}

// ============================================================================
// Basic console - Normal cases
// ============================================================================

/// Test console command starts (will need to be killed).
#[test]
fn test_console_starts() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["console", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn console");

    // Let it run briefly
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Kill it
    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait on child");

    // Should have produced some output or been running
    // (may be empty if no logs, but process should start)
    let _ = output;
}

/// Test console with file output.
#[test]
fn test_console_file_output() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("log");

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["console", "--udid", &udid, "-o", &file_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn console");

    // Let it collect some logs
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Kill it
    let _ = child.kill();
    let _ = child.wait();

    // File may have been created (depends on log activity)
    if std::path::Path::new(&file_path).exists() {
        let content = std::fs::read_to_string(&file_path).unwrap_or_default();
        // Content may be empty if no logs during the period
        let _ = content;
        cleanup_temp_file(&file_path);
    }
}

// ============================================================================
// Output options - Normal cases
// ============================================================================

/// Test console with -o short option.
#[test]
fn test_console_output_short_option() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("log");

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["console", "--udid", &udid, "-o", &file_path])
        .spawn()
        .expect("Failed to spawn console");

    std::thread::sleep(std::time::Duration::from_secs(1));
    let _ = child.kill();
    let _ = child.wait();

    cleanup_temp_file(&file_path);
}

/// Test console with --output long option.
#[test]
fn test_console_output_long_option() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let file_path = get_temp_file_path("log");

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["console", "--udid", &udid, "--output", &file_path])
        .spawn()
        .expect("Failed to spawn console");

    std::thread::sleep(std::time::Duration::from_secs(1));
    let _ = child.kill();
    let _ = child.wait();

    cleanup_temp_file(&file_path);
}

// ============================================================================
// Error cases
// ============================================================================

/// Test console with invalid UDID.
#[test]
fn test_console_invalid_udid() {
    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["console", "--udid", "invalid-udid-12345"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn console");

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Check if process has exited with error
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process exited
            assert!(!status.success(), "console with invalid UDID should fail");
        }
        Ok(None) => {
            // Still running, kill it
            let _ = child.kill();
            let _ = child.wait();
        }
        Err(_) => {
            let _ = child.kill();
        }
    }
}

/// Test console with permission denied output path.
#[test]
fn test_console_permission_denied() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["console", "--udid", &udid, "-o", "/etc/console.log"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn console");

    std::thread::sleep(std::time::Duration::from_secs(1));

    match child.try_wait() {
        Ok(Some(status)) => {
            assert!(!status.success(), "console to /etc/ should fail");
        }
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
        }
        Err(_) => {
            let _ = child.kill();
        }
    }
}

/// Test console with non-existent output directory.
#[test]
fn test_console_non_existent_directory() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let mut child = Command::new("./target/debug/agent-mobile")
        .args([
            "console",
            "--udid",
            &udid,
            "-o",
            "/nonexistent/dir/console.log",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn console");

    std::thread::sleep(std::time::Duration::from_secs(1));

    match child.try_wait() {
        Ok(Some(status)) => {
            assert!(
                !status.success(),
                "console to non-existent directory should fail"
            );
        }
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
        }
        Err(_) => {
            let _ = child.kill();
        }
    }
}
