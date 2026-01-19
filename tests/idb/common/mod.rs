#![allow(dead_code)]

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::{Child, Command, Output, Stdio};
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

/// Parse multi-path ls output into sections (path -> files)
fn parse_ls_sections(output: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut sections = std::collections::BTreeMap::new();
    let mut current_path: Option<String> = None;
    let mut current_files: Vec<String> = Vec::new();

    for line in output.lines() {
        if line.ends_with(':') {
            // Save previous section
            if let Some(path) = current_path.take() {
                current_files.sort();
                sections.insert(path, current_files);
                current_files = Vec::new();
            }
            // Start new section
            current_path = Some(line.trim_end_matches(':').to_string());
        } else if !line.is_empty() {
            current_files.push(line.to_string());
        }
    }

    // Save last section
    if let Some(path) = current_path {
        current_files.sort();
        sections.insert(path, current_files);
    }

    sections
}

/// Compare multi-path ls outputs (order-independent comparison)
///
/// For multi-path ls commands, the gRPC response order may vary.
/// This function parses sections and compares them regardless of order.
pub fn compare_file_ls_multi_path_outputs(python_output: &Output, rust_output: &Output) {
    // Compare exit codes
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ:\n  Python idb: {:?}\n  agent-mobile: {:?}",
        python_output.status.code(),
        rust_output.status.code()
    );

    let python_stdout = String::from_utf8_lossy(&python_output.stdout);
    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

    // Parse and compare sections
    let python_sections = parse_ls_sections(&python_stdout);
    let rust_sections = parse_ls_sections(&rust_stdout);

    assert_eq!(
        python_sections,
        rust_sections,
        "ls output sections differ:\n  Python idb paths: {:?}\n  agent-mobile paths: {:?}",
        python_sections.keys().collect::<Vec<_>>(),
        rust_sections.keys().collect::<Vec<_>>()
    );

    // Check stderr/success status
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

/// Run Python idb command with given arguments
pub fn run_idb_command(args: &[&str]) -> Output {
    Command::new("idb")
        .args(args)
        .output()
        .expect("Failed to execute Python idb - ensure idb is installed and in PATH")
}

/// Run agent-mobile idb command with given arguments
pub fn run_agent_mobile_command(args: &[&str]) -> Output {
    Command::new("./target/debug/agent-mobile")
        .args(args)
        .output()
        .expect("Failed to run agent-mobile - ensure it is built with 'cargo build'")
}

/// Compare outputs from Python idb and agent-mobile (generic version)
pub fn compare_command_outputs(python_output: &Output, rust_output: &Output) {
    // Compare exit codes
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ:\n  Python idb: {:?}\n  agent-mobile: {:?}",
        python_output.status.code(),
        rust_output.status.code()
    );

    // For success cases, compare stdout
    if python_output.status.success() {
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
        assert_eq!(
            python_stdout.trim(),
            rust_stdout.trim(),
            "stdout differs:\n  Python idb:\n{}\n  agent-mobile:\n{}",
            python_stdout,
            rust_stdout
        );
    }
}

/// Ensure the simulator's framebuffer is initialized and ready for screenshots.
///
/// This function directly attempts to take a screenshot until it succeeds, which is
/// the most reliable way to verify framebuffer initialization. The framebuffer/IOSurface
/// may not be initialized immediately after companion startup, so we retry with delays.
///
/// Note: This function does not guarantee framebuffer readiness, as initialization is
/// environment-dependent. Tests should handle "No Image available to encode" errors
/// gracefully by skipping when appropriate.
///
/// Should be called before screenshot tests or any operations requiring framebuffer access.
pub fn ensure_framebuffer_ready(udid: &str) {
    const MAX_ATTEMPTS: u32 = 10;
    const RETRY_DELAY_MS: u64 = 1000;

    ensure_companion_running(udid);

    // スクリーンショットを直接試行して、フレームバッファが準備できているか確認
    for attempt in 1..=MAX_ATTEMPTS {
        let output = Command::new("idb")
            .args(["screenshot", "--udid", udid, "-"])
            .stdout(Stdio::null()) // 画像データを破棄
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute idb screenshot");

        if output.status.success() {
            eprintln!("Framebuffer ready after {} attempt(s)", attempt);
            return;
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No Image available to encode") {
            if attempt < MAX_ATTEMPTS {
                eprintln!(
                    "Attempt {}/{}: Framebuffer not ready, retrying...",
                    attempt, MAX_ATTEMPTS
                );
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }
        } else {
            // 異なるエラー - 警告を出して終了
            eprintln!(
                "Warning: Failed to initialize framebuffer (unexpected error): {}",
                stderr
            );
            return;
        }
    }

    // フレームバッファが初期化できなかったが、テストレベルでスキップ処理が行われる
    eprintln!(
        "Warning: Framebuffer failed to initialize after {} attempts. Tests may be skipped.",
        MAX_ATTEMPTS
    );
}
