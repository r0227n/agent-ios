use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

use super::client::XCUITestClient;

/// Bundle ID for the XCUITest Runner host app.
const RUNNER_HOST_BUNDLE_ID: &str = "com.agent-mobile.xcuitest-runner";

/// Bundle ID for the XCUITest Runner test runner (xctrunner).
const RUNNER_XCTRUNNER_BUNDLE_ID: &str = "com.agent-mobile.xcuitest-runner-uitests.xctrunner";

/// Errors that can occur when starting the XCUITest Runner.
#[derive(Debug, thiserror::Error)]
pub enum RunnerStartError {
    #[error("XCUITest Runner project not found")]
    ProjectNotFound,
    #[error("No booted simulator found: {0}")]
    NoBootedSimulator(String),
    #[error("Build failed: {0}")]
    BuildFailed(String),
<<<<<<< HEAD
    #[error("Build products not found (searched: {0})")]
    BuildProductsNotFound(String),
    #[error("App install failed: {0}")]
    InstallFailed(String),
    #[error("App launch failed: {0}")]
    LaunchFailed(String),
    #[error("Health check timed out after {0} seconds")]
    HealthCheckTimeout(u64),
||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
=======
    #[error("Build products not found (searched: {0})")]
    BuildProductsNotFound(String),
    #[error("App install failed: {0}")]
    InstallFailed(String),
    #[error("App launch failed: {0}")]
    LaunchFailed(String),
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
}

/// Pre-built .app bundle paths for the XCUITest Runner.
pub struct RunnerBuildProducts {
    /// Path to XCUITestRunner.app (host app)
    pub host_app: PathBuf,
    /// Path to XCUITestRunnerUITests-Runner.app (test runner)
    pub runner_app: PathBuf,
}

impl RunnerBuildProducts {
    /// Search for pre-built .app bundles in standard locations.
    ///
    /// Search order:
    /// 1. Adjacent to the current binary (installed layout)
    /// 2. DerivedData (Xcode build output)
    /// 3. Relative to CWD (development layout)
    pub fn find() -> Option<Self> {
        // 1. Adjacent to current binary
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let products = Self::check_dir(parent);
                if products.is_some() {
                    return products;
                }
            }
        }

        // 2. DerivedData — search for the most recently modified build directory
        if let Some(products) = Self::search_derived_data() {
            return Some(products);
        }

        // 3. Relative to CWD
        if let Ok(cwd) = std::env::current_dir() {
            let build_dir = cwd.join("build/Build/Products/Debug-iphonesimulator");
            let products = Self::check_dir(&build_dir);
            if products.is_some() {
                return products;
            }
        }

        None
    }

    /// Check a directory for both required .app bundles.
    fn check_dir(dir: &Path) -> Option<Self> {
        let host_app = dir.join("XCUITestRunner.app");
        let runner_app = dir.join("XCUITestRunnerUITests-Runner.app");

        if host_app.exists() && runner_app.exists() {
            Some(Self {
                host_app,
                runner_app,
            })
        } else {
            None
        }
    }

    /// Search DerivedData for build products.
    fn search_derived_data() -> Option<Self> {
        let home = std::env::var("HOME").ok()?;
        let derived_data = PathBuf::from(home).join("Library/Developer/Xcode/DerivedData");

        if !derived_data.exists() {
            return None;
        }

        // Find XCUITestRunner-* directories
        let mut candidates: Vec<PathBuf> = std::fs::read_dir(&derived_data)
            .ok()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with("XCUITestRunner-"))
            })
            .map(|entry| entry.path().join("Build/Products/Debug-iphonesimulator"))
            .collect();

        // Sort by modification time, most recent first
        candidates.sort_by(|a, b| {
            let time_a = std::fs::metadata(a)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let time_b = std::fs::metadata(b)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            time_b.cmp(&time_a)
        });

        for dir in candidates {
            if let Some(products) = Self::check_dir(&dir) {
                return Some(products);
            }
        }

        None
    }

    /// Describe the paths that were searched (for error messages).
    pub fn searched_paths_description() -> String {
        let mut paths = Vec::new();

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                paths.push(parent.display().to_string());
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            paths.push(format!(
                "{}/Library/Developer/Xcode/DerivedData/XCUITestRunner-*/Build/Products/Debug-iphonesimulator",
                home
            ));
        }

        if let Ok(cwd) = std::env::current_dir() {
            paths.push(
                cwd.join("build/Build/Products/Debug-iphonesimulator")
                    .display()
                    .to_string(),
            );
        }

        paths.join(", ")
    }
}

/// Manages the lifecycle of the XCUITest Runner.
///
/// After the switch to CoreSimulator FFI, the runner is managed through
/// install/launch/terminate rather than a child process.
pub struct XCUITestRunner {
    /// UDID of the target simulator
    udid: String,
    /// Port for the HTTP server
    port: u16,
}

impl XCUITestRunner {
    pub fn new(udid: String, port: u16) -> Self {
        Self { udid, port }
    }

    /// Locate the bundled XCUITestRunner project relative to the agent-mobile binary.
    pub fn bundled_project_path() -> Option<PathBuf> {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let dev_path = parent
                    .parent()
                    .unwrap_or(parent)
                    .join("crates/xcuitest-runner/XCUITestRunner.xcodeproj");
                if dev_path.exists() {
                    return Some(dev_path);
                }
            }
        }

        let cwd_path = PathBuf::from("crates/xcuitest-runner/XCUITestRunner.xcodeproj");
        if cwd_path.exists() {
            return Some(cwd_path);
        }

        None
    }

    /// Stop the XCUITest Runner via CoreSimulator terminate.
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Terminate the xctrunner (test runner app)
        let _ = crate::coresim::terminate_app(&self.udid, RUNNER_XCTRUNNER_BUNDLE_ID);
        // Also terminate the host app
        let _ = crate::coresim::terminate_app(&self.udid, RUNNER_HOST_BUNDLE_ID);
        Ok(())
    }

    /// Get a client connected to this runner.
    pub fn client(&self) -> XCUITestClient {
        XCUITestClient::new(self.port)
    }
}

impl Drop for XCUITestRunner {
    fn drop(&mut self) {
        let _ = crate::coresim::terminate_app(&self.udid, RUNNER_XCTRUNNER_BUNDLE_ID);
        let _ = crate::coresim::terminate_app(&self.udid, RUNNER_HOST_BUNDLE_ID);
    }
}

/// Start the XCUITest Runner via CoreSimulator FFI (install + launch).
///
/// Installs both the host app and test runner app, then launches the test runner.
/// Polls for health check readiness after launch.
pub async fn start_runner_detached(
    products: &RunnerBuildProducts,
    udid: &str,
    port: u16,
) -> Result<(), RunnerStartError> {
    // Install host app
    crate::coresim::install_app(udid, &products.host_app)
        .map_err(|e| RunnerStartError::InstallFailed(format!("host app: {}", e)))?;

    // Install test runner app
    crate::coresim::install_app(udid, &products.runner_app)
        .map_err(|e| RunnerStartError::InstallFailed(format!("runner app: {}", e)))?;

<<<<<<< HEAD
    // Launch the test runner
    let pid = crate::coresim::launch_app(udid, RUNNER_XCTRUNNER_BUNDLE_ID)
        .map_err(|e| RunnerStartError::LaunchFailed(format!("{}", e)))?;
    info!(pid, "XCUITest Runner launched");
||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
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
=======
    // Launch the test runner
    let pid = crate::coresim::launch_app(udid, RUNNER_XCTRUNNER_BUNDLE_ID)
        .map_err(|e| RunnerStartError::LaunchFailed(format!("{}", e)))?;
    eprintln!("XCUITest Runner launched (PID: {pid})");
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)

    // Poll for readiness
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(120);
    let retry_interval = Duration::from_millis(500);
    let progress_interval = Duration::from_secs(10);
    let mut last_progress = std::time::Instant::now();
    let client = XCUITestClient::new(port);

    loop {
        if client.health_check().await.unwrap_or(false) {
            return Ok(());
        }

<<<<<<< HEAD
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return Err(RunnerStartError::HealthCheckTimeout(timeout.as_secs()));
        }

||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
        // Show progress every 10 seconds so the user knows we're still waiting.
=======
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
        if last_progress.elapsed() >= progress_interval {
            info!(
                elapsed_secs = elapsed.as_secs(),
                "Still waiting for XCUITest Runner to become ready"
            );
            last_progress = std::time::Instant::now();
        }

        tokio::time::sleep(retry_interval).await;
    }
}

/// Build the XCUITest Runner using `xcodebuild build-for-testing`.
///
/// This is a fallback when pre-built products are not found.
pub async fn build_for_testing(project_path: &Path, udid: &str) -> Result<(), RunnerStartError> {
    let project_str = project_path
        .to_str()
        .ok_or(RunnerStartError::ProjectNotFound)?;

    info!("Building XCUITest Runner (this may take a moment)...");

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
        .map_err(|e| RunnerStartError::BuildFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
<<<<<<< HEAD
        let truncated = if stderr.len() > 2000 {
            let end = stderr
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|&i| i <= 2000)
                .last()
                .unwrap_or(0);
            format!("{}... (truncated)", &stderr[..end])
        } else {
            stderr.to_string()
        };
        return Err(RunnerStartError::BuildFailed(truncated));
||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
        return Err(RunnerStartError::BuildFailed(truncate_stderr(
            &stderr, 2000,
        )));
=======
        let truncated = if stderr.len() > 2000 {
            format!("{}... (truncated)", &stderr[..2000])
        } else {
            stderr.to_string()
        };
        return Err(RunnerStartError::BuildFailed(truncated));
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
    }

    info!("XCUITest Runner build completed successfully");
    Ok(())
}

/// Ensure the XCUITest Runner is started and ready to accept connections.
///
/// Flow:
/// 1. If the runner is already healthy, returns immediately.
/// 2. Looks for pre-built products and installs/launches via CoreSimulator.
/// 3. If products not found, falls back to `xcodebuild build-for-testing` and retries.
pub async fn ensure_runner_started(port: u16) -> Result<(), RunnerStartError> {
    let client = XCUITestClient::new(port);

    // Fast path: runner is already up.
    if client.health_check().await.unwrap_or(false) {
        return Ok(());
    }

    info!("XCUITest Runner is not running. Starting automatically...");

<<<<<<< HEAD
    let booted = crate::coresim::get_booted_device()
        .map_err(|e| RunnerStartError::NoBootedSimulator(e.to_string()))?;

    // Try pre-built products first
    if let Some(products) = RunnerBuildProducts::find() {
        match start_runner_detached(&products, &booted.udid, port).await {
            Ok(()) => {
                info!(simulator = %booted.name, "XCUITest Runner started successfully");
                return Ok(());
            }
            Err(e) => {
                warn!(%e, "Failed to start from pre-built products. Falling back to build...");
            }
        }
    }

    // Fallback: build and retry
||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
=======
    let booted = crate::coresim::get_booted_device()
        .map_err(|e| RunnerStartError::NoBootedSimulator(e.to_string()))?;

    // Try pre-built products first
    if let Some(products) = RunnerBuildProducts::find() {
        match start_runner_detached(&products, &booted.udid, port).await {
            Ok(()) => {
                eprintln!(
                    "XCUITest Runner started successfully on simulator '{}'.",
                    booted.name
                );
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to start from pre-built products: {e}. Falling back to build...");
            }
        }
    }

    // Fallback: build and retry
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
    let project_path =
        XCUITestRunner::bundled_project_path().ok_or(RunnerStartError::ProjectNotFound)?;

    build_for_testing(&project_path, &booted.udid).await?;

    // After building, products should be available in DerivedData
    let products = RunnerBuildProducts::find().ok_or_else(|| {
        RunnerStartError::BuildProductsNotFound(RunnerBuildProducts::searched_paths_description())
    })?;

<<<<<<< HEAD
    start_runner_detached(&products, &booted.udid, port).await?;
    info!(simulator = %booted.name, "XCUITest Runner started successfully");
    Ok(())
||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
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
=======
    start_runner_detached(&products, &booted.udid, port).await?;
    eprintln!(
        "XCUITest Runner started successfully on simulator '{}'.",
        booted.name
    );
    Ok(())
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
}
