use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;

use super::client::XCUITestClient;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Errors that can occur when starting the XCUITest Runner.
#[derive(Debug, thiserror::Error)]
pub enum RunnerStartError {
    #[error(
        "XCUITest Runner process exited early (exit code: {exit_code:?}):\n{stderr}\nHint: {hint}"
    )]
    ProcessExitedEarly {
        exit_code: Option<i32>,
        stderr: String,
        hint: String,
    },
    #[error("Failed to spawn xcodebuild: {0}")]
    SpawnFailed(#[source] std::io::Error),
    #[error("XCUITest Runner project not found")]
    ProjectNotFound,
    #[error("No booted simulator found: {0}")]
    NoBootedSimulator(String),
    #[error("Build failed: {0}")]
    BuildFailed(String),
}

/// Manages the lifecycle of the XCUITest Runner process.
pub struct XCUITestRunner {
    /// Path to the XCUITestRunner.xcodeproj
    project_path: PathBuf,
    /// Destination simulator name or UDID
    destination: String,
    /// Port for the HTTP server
    port: u16,
    /// Child process handle
    process: Option<tokio::process::Child>,
}

impl XCUITestRunner {
    /// Create a new runner manager.
    pub fn new(project_path: PathBuf, destination: String, port: u16) -> Self {
        Self {
            project_path,
            destination,
            port,
            process: None,
        }
    }

    /// Locate the bundled XCUITestRunner project relative to the agent-mobile binary.
    pub fn bundled_project_path() -> Option<PathBuf> {
        // Try to find the project relative to the current executable
        if let Ok(exe) = std::env::current_exe() {
            // Standard install: binary at /path/to/bin/agent-mobile
            // Project at: /path/to/share/xcuitest-runner/XCUITestRunner.xcodeproj
            if let Some(parent) = exe.parent() {
                // Check sibling crates directory (development layout)
                let dev_path = parent
                    .parent()
                    .unwrap_or(parent)
                    .join("crates/xcuitest-runner/XCUITestRunner.xcodeproj");
                if dev_path.exists() {
                    return Some(dev_path);
                }
            }
        }

        // Fallback: check relative to CWD
        let cwd_path = PathBuf::from("crates/xcuitest-runner/XCUITestRunner.xcodeproj");
        if cwd_path.exists() {
            return Some(cwd_path);
        }

        None
    }

    /// Start the XCUITest Runner via xcodebuild test-without-building.
    pub async fn start(&mut self) -> Result<()> {
        let project_str = self.project_path.to_str().ok_or("Invalid project path")?;

        let child = Command::new("xcodebuild")
            .args([
                "test-without-building",
                "-project",
                project_str,
                "-scheme",
                "XCUITestRunner",
                "-destination",
                &format!("platform=iOS Simulator,name={}", self.destination),
                "-only-testing",
                "XCUITestRunnerUITests/AutomationServer/testStartAutomationServer",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        self.process = Some(child);

        // Wait for the server to be ready
        let client = XCUITestClient::new(self.port);
        client.wait_for_ready_indefinitely().await?;

        Ok(())
    }

    /// Stop the XCUITest Runner.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.process {
            child.kill().await?;
            self.process = None;
        }
        Ok(())
    }

    /// Check if the runner is currently running.
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(None) => true, // Still running
                _ => false,       // Exited or error
            }
        } else {
            false
        }
    }

    /// Get a client connected to this runner.
    pub fn client(&self) -> XCUITestClient {
        XCUITestClient::new(self.port)
    }
}

/// Start the XCUITest Runner as a detached background process.
///
/// The spawned process continues running after the calling CLI process exits,
/// because `std::process::Child` does not kill the child on drop.
/// stderr is captured via a temp file so early failures can be reported.
pub async fn start_runner_detached(
    project_path: &Path,
    udid: &str,
    port: u16,
) -> std::result::Result<(), RunnerStartError> {
    let project_str = project_path.to_str().ok_or_else(|| {
        RunnerStartError::SpawnFailed(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid project path",
        ))
    })?;

    // Capture stderr in a temp file so we can read it if the process exits early.
    let stderr_file = tempfile::NamedTempFile::new().map_err(RunnerStartError::SpawnFailed)?;
    let stderr_for_child = stderr_file
        .reopen()
        .map_err(RunnerStartError::SpawnFailed)?;

    // Use std::process::Command (not tokio) so the child is fully detached.
    let mut child = std::process::Command::new("xcodebuild")
        .args([
            "test-without-building",
            "-project",
            project_str,
            "-scheme",
            "XCUITestRunner",
            "-destination",
            &format!("id={}", udid),
            "-only-testing",
            "XCUITestRunnerUITests/AutomationServer/testStartAutomationServer",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::from(stderr_for_child))
        .spawn()
        .map_err(RunnerStartError::SpawnFailed)?;

    // Poll for readiness, checking for early process exit at each iteration.
    // No timeout — waits indefinitely until the runner is ready or the process exits.
    let start = std::time::Instant::now();
    let retry_interval = Duration::from_millis(500);
    let progress_interval = Duration::from_secs(10);
    let mut last_progress = std::time::Instant::now();
    let client = XCUITestClient::new(port);

    loop {
        // Check if the process has already exited (early failure).
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = read_stderr_file(&stderr_file);
                let hint = classify_xcodebuild_error(&stderr);
                return Err(RunnerStartError::ProcessExitedEarly {
                    exit_code: status.code(),
                    stderr: truncate_stderr(&stderr, 2000),
                    hint,
                });
            }
            Ok(None) => { /* still running, good */ }
            Err(e) => return Err(RunnerStartError::SpawnFailed(e)),
        }

        // Check health.
        if client.health_check().await.unwrap_or(false) {
            return Ok(());
        }

        // Show progress every 10 seconds so the user knows we're still waiting.
        if last_progress.elapsed() >= progress_interval {
            let elapsed = start.elapsed().as_secs();
            eprintln!("Still waiting for XCUITest Runner to become ready ({elapsed}s elapsed)...");
            last_progress = std::time::Instant::now();
        }

        tokio::time::sleep(retry_interval).await;
    }
}

/// Build the XCUITest Runner using `xcodebuild build-for-testing`.
pub async fn build_for_testing(
    project_path: &Path,
    udid: &str,
) -> std::result::Result<(), RunnerStartError> {
    let project_str = project_path.to_str().ok_or_else(|| {
        RunnerStartError::SpawnFailed(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid project path",
        ))
    })?;

    eprintln!("Building XCUITest Runner (this may take a moment)...");

    let output = Command::new("xcodebuild")
        .args([
            "build-for-testing",
            "-project",
            project_str,
            "-scheme",
            "XCUITestRunner",
            "-destination",
            &format!("id={}", udid),
        ])
        .output()
        .await
        .map_err(RunnerStartError::SpawnFailed)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RunnerStartError::BuildFailed(truncate_stderr(
            &stderr, 2000,
        )));
    }

    eprintln!("XCUITest Runner build completed successfully.");
    Ok(())
}

/// Ensure the XCUITest Runner is started and ready to accept connections.
///
/// This is the main entry point for automatic runner management:
/// 1. If the runner is already healthy, returns immediately.
/// 2. Tries `test-without-building` — if it fails because the product
///    hasn't been built yet, runs `build-for-testing` and retries.
/// 3. Returns a descriptive error if the runner cannot be started.
pub async fn ensure_runner_started(port: u16) -> std::result::Result<(), RunnerStartError> {
    let client = XCUITestClient::new(port);

    // Fast path: runner is already up.
    if client.health_check().await.unwrap_or(false) {
        return Ok(());
    }

    eprintln!("XCUITest Runner is not running. Starting automatically...");

    let project_path =
        XCUITestRunner::bundled_project_path().ok_or(RunnerStartError::ProjectNotFound)?;

    let booted = crate::simctl::get_booted_simulator()
        .map_err(|e| RunnerStartError::NoBootedSimulator(e.to_string()))?;

    match start_runner_detached(&project_path, &booted.udid, port).await {
        Ok(()) => {
            eprintln!(
                "XCUITest Runner started successfully on simulator '{}'.",
                booted.name
            );
            Ok(())
        }
        Err(RunnerStartError::ProcessExitedEarly { ref stderr, .. }) if needs_build(stderr) => {
            eprintln!("XCUITest Runner not built yet. Building automatically...");
            build_for_testing(&project_path, &booted.udid).await?;

            // Retry after building.
            start_runner_detached(&project_path, &booted.udid, port).await?;
            eprintln!(
                "XCUITest Runner started successfully on simulator '{}'.",
                booted.name
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Classify xcodebuild stderr output and return a user-friendly hint.
pub(crate) fn classify_xcodebuild_error(stderr: &str) -> String {
    if needs_build(stderr) {
        return "Run `xcodebuild build-for-testing` first, or let ensure_runner_started() handle it automatically.".to_string();
    }
    if stderr.contains("Unable to find a destination matching the provided destination specifier")
        || stderr.contains("device not found")
    {
        return "No matching simulator found. Boot a simulator with `xcrun simctl boot <UDID>`."
            .to_string();
    }
    if stderr.contains("xcodebuild: error: Could not resolve package dependencies") {
        return "Swift Package Manager dependency resolution failed. Check your network connection.".to_string();
    }
    if stderr.contains("is not a project file") || stderr.contains("does not exist") {
        return "XCUITest Runner project not found at the expected path.".to_string();
    }
    "Check xcodebuild output above for details.".to_string()
}

/// Truncate stderr output to at most `max_chars` characters.
pub(crate) fn truncate_stderr(stderr: &str, max_chars: usize) -> String {
    if stderr.len() <= max_chars {
        stderr.to_string()
    } else {
        let truncated = &stderr[..max_chars];
        format!("{}... (truncated)", truncated)
    }
}

/// Check whether the xcodebuild error indicates the test bundle has not been built.
pub(crate) fn needs_build(stderr: &str) -> bool {
    // xcodebuild test-without-building fails with these messages when not built
    stderr.contains("xcodebuild: error: Failed to build")
        || stderr.contains("no test bundle found")
        || stderr.contains("Could not find test runner")
        || stderr.contains("Test product not found")
        || stderr.contains("cannot be located")
        || stderr.contains("doesn't contain a test host")
}

/// Read stderr content from the temp file.
fn read_stderr_file(file: &tempfile::NamedTempFile) -> String {
    let mut f = match file.reopen() {
        Ok(f) => f,
        Err(_) => return String::new(),
    };
    let mut content = String::new();
    let _ = f.read_to_string(&mut content);
    content
}

impl Drop for XCUITestRunner {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.process {
            let _ = child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_needs_build() {
        let stderr = "xcodebuild: error: Failed to build something";
        let hint = classify_xcodebuild_error(stderr);
        assert!(hint.contains("build-for-testing"));
    }

    #[test]
    fn test_classify_no_test_bundle() {
        let stderr = "error: no test bundle found at path";
        let hint = classify_xcodebuild_error(stderr);
        assert!(hint.contains("build-for-testing"));
    }

    #[test]
    fn test_classify_no_simulator() {
        let stderr = "Unable to find a destination matching the provided destination specifier";
        let hint = classify_xcodebuild_error(stderr);
        assert!(hint.contains("Boot a simulator"));
    }

    #[test]
    fn test_classify_device_not_found() {
        let stderr = "error: device not found for id=ABCD-1234";
        let hint = classify_xcodebuild_error(stderr);
        assert!(hint.contains("Boot a simulator"));
    }

    #[test]
    fn test_classify_package_deps() {
        let stderr = "xcodebuild: error: Could not resolve package dependencies";
        let hint = classify_xcodebuild_error(stderr);
        assert!(hint.contains("network connection"));
    }

    #[test]
    fn test_classify_project_not_found() {
        let stderr = "error: '/tmp/foo.xcodeproj' is not a project file";
        let hint = classify_xcodebuild_error(stderr);
        assert!(hint.contains("project not found"));
    }

    #[test]
    fn test_classify_unknown_error() {
        let stderr = "some completely unknown error";
        let hint = classify_xcodebuild_error(stderr);
        assert!(hint.contains("Check xcodebuild output"));
    }

    #[test]
    fn test_truncate_stderr_short() {
        let s = "short message";
        assert_eq!(truncate_stderr(s, 100), "short message");
    }

    #[test]
    fn test_truncate_stderr_exact() {
        let s = "12345";
        assert_eq!(truncate_stderr(s, 5), "12345");
    }

    #[test]
    fn test_truncate_stderr_long() {
        let s = "a".repeat(200);
        let result = truncate_stderr(&s, 50);
        assert!(result.len() < 200);
        assert!(result.ends_with("... (truncated)"));
        // 50 chars of 'a' + "... (truncated)"
        assert!(result.starts_with(&"a".repeat(50)));
    }

    #[test]
    fn test_needs_build_positive() {
        assert!(needs_build("xcodebuild: error: Failed to build workspace"));
        assert!(needs_build("error: no test bundle found at /path"));
        assert!(needs_build("Could not find test runner for XCUITestRunner"));
        assert!(needs_build("Test product not found"));
        assert!(needs_build("the test bundle cannot be located"));
    }

    #[test]
    fn test_needs_build_negative() {
        assert!(!needs_build("Build succeeded"));
        assert!(!needs_build("Test session started"));
        assert!(!needs_build("some random error"));
    }

    #[test]
    fn test_runner_start_error_display() {
        let err = RunnerStartError::ProjectNotFound;
        assert!(err.to_string().contains("not found"));

        let err = RunnerStartError::ProcessExitedEarly {
            exit_code: Some(65),
            stderr: "test error".to_string(),
            hint: "build first".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("65"));
        assert!(msg.contains("test error"));
        assert!(msg.contains("build first"));
    }
}
