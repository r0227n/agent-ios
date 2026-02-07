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
    #[error("Build products not found (searched: {0})")]
    BuildProductsNotFound(String),
    #[error("App install failed: {0}")]
    InstallFailed(String),
    #[error("App launch failed: {0}")]
    LaunchFailed(String),
    #[error("Health check timed out after {0} seconds")]
    HealthCheckTimeout(u64),
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

    // Launch the test runner
    let pid = crate::coresim::launch_app(udid, RUNNER_XCTRUNNER_BUNDLE_ID)
        .map_err(|e| RunnerStartError::LaunchFailed(format!("{}", e)))?;
    info!(pid, "XCUITest Runner launched");

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

        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return Err(RunnerStartError::HealthCheckTimeout(timeout.as_secs()));
        }

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
    let project_path =
        XCUITestRunner::bundled_project_path().ok_or(RunnerStartError::ProjectNotFound)?;

    build_for_testing(&project_path, &booted.udid).await?;

    // After building, products should be available in DerivedData
    let products = RunnerBuildProducts::find().ok_or_else(|| {
        RunnerStartError::BuildProductsNotFound(RunnerBuildProducts::searched_paths_description())
    })?;

    start_runner_detached(&products, &booted.udid, port).await?;
    info!(simulator = %booted.name, "XCUITest Runner started successfully");
    Ok(())
}
