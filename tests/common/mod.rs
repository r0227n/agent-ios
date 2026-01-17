#![allow(dead_code)]

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::{Child, Command, Output};
use std::thread;
use std::time::{Duration, Instant};

/// Get a valid UDID from idb list-targets (prefers Booted simulator)
///
/// Returns the UDID of the first Booted simulator.
/// Panics if no booted simulator is available.
pub fn get_available_udid() -> String {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .expect("Failed to execute Python idb - ensure idb is installed and in PATH");

    assert!(
        output.status.success(),
        "Python idb list-targets failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if json.get("state").and_then(|v| v.as_str()) == Some("Booted") {
                return json.get("udid").unwrap().as_str().unwrap().to_string();
            }
        }
    }
    panic!("No booted simulator available");
}

/// Ensure companion is running for the given UDID by executing a Python idb command.
/// This triggers automatic companion startup.
pub fn ensure_companion_running(udid: &str) {
    // Running idb describe with --udid will start the companion if needed
    let output = Command::new("idb")
        .args(["describe", "--udid", udid])
        .output()
        .expect("Failed to execute Python idb");

    assert!(
        output.status.success(),
        "Failed to ensure companion is running: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Get the path to the mock app for testing
pub fn get_mock_app_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/fixtures/MockApp.app", manifest_dir)
}

/// Get the test app path (mock app or custom via env var)
pub fn get_test_app_path() -> String {
    std::env::var("TEST_APP_PATH").unwrap_or_else(|_| get_mock_app_path())
}

/// Get installed app bundle ID for testing (Settings app is always available)
pub fn get_test_bundle_id() -> String {
    "com.apple.Preferences".to_string()
}

/// Wait for child process with timeout, killing if necessary to prevent hangs
pub fn wait_with_timeout(mut child: Child, timeout_secs: u64) -> Output {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    // First, send SIGTERM after initial wait period
    thread::sleep(Duration::from_secs(2));

    #[cfg(unix)]
    {
        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
    }

    // Poll for process exit with timeout
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process exited
                return child.wait_with_output().expect("Failed to get output");
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    // Timeout - force kill
                    let _ = child.kill();
                    return child
                        .wait_with_output()
                        .expect("Failed to get output after kill");
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                panic!("Error waiting for process: {}", e);
            }
        }
    }
}

/// Run Python idb file command with given arguments
///
/// Example: run_idb_file_command(&["ls", "/tmp", "--udid", "ABC123"])
pub fn run_idb_file_command(args: &[&str]) -> Output {
    let mut full_args = vec!["file"];
    full_args.extend_from_slice(args);

    Command::new("idb")
        .args(&full_args)
        .output()
        .expect("Failed to execute Python idb - ensure idb is installed and in PATH")
}

/// Run agent-mobile idb file command with given arguments
///
/// Example: run_agent_mobile_file_command(&["ls", "/tmp", "--udid", "ABC123"])
pub fn run_agent_mobile_file_command(args: &[&str]) -> Output {
    let mut full_args = vec!["idb", "file"];
    full_args.extend_from_slice(args);

    Command::new("./target/debug/agent-mobile")
        .args(&full_args)
        .output()
        .expect("Failed to run agent-mobile - ensure it is built with 'cargo build'")
}

/// Compare outputs from Python idb and agent-mobile
///
/// Compares exit codes and stdout/stderr content
pub fn compare_file_command_outputs(python_output: &Output, rust_output: &Output) {
    // Compare exit codes
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ:\n  Python idb: {:?}\n  agent-mobile: {:?}",
        python_output.status.code(),
        rust_output.status.code()
    );

    // Compare stdout
    let python_stdout = String::from_utf8_lossy(&python_output.stdout);
    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
    assert_eq!(
        python_stdout.trim(),
        rust_stdout.trim(),
        "stdout differs:\n  Python idb:\n{}\n  agent-mobile:\n{}",
        python_stdout,
        rust_stdout
    );

    // Note: stderr comparison is relaxed as error messages may have minor formatting differences
    // We only check that both have errors or both succeed
    let python_has_error = !python_output.stderr.is_empty();
    let rust_has_error = !rust_output.stderr.is_empty();

    if python_output.status.success() {
        assert!(
            rust_output.status.success(),
            "Python succeeded but Rust failed:\n{}",
            String::from_utf8_lossy(&rust_output.stderr)
        );
    } else {
        assert!(
            !rust_output.status.success(),
            "Python failed but Rust succeeded"
        );
    }
}

/// Build agent-mobile binary (debug mode)
pub fn build_agent_mobile() {
    let output = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to run cargo build");

    assert!(
        output.status.success(),
        "cargo build failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
