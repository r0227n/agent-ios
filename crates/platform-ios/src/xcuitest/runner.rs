use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tracing::info;

use super::client::XCUITestClient;

/// Bundle ID for the XCUITest Runner host app.
const RUNNER_HOST_BUNDLE_ID: &str = "com.agent-mobile.xcuitest-runner";

/// Bundle ID for the XCUITest Runner test runner (xctrunner).
const RUNNER_XCTRUNNER_BUNDLE_ID: &str = "com.agent-mobile.xcuitest-runner-uitests.xctrunner";
/// Specific UI test entrypoint that keeps the automation server alive.
const RUNNER_TEST_ENTRYPOINT: &str =
    "XCUITestRunnerUITests/AutomationServer/testStartAutomationServer";
/// Max time to wait for `/ready` after spawning xcodebuild.
const READY_TIMEOUT: Duration = Duration::from_secs(30);
/// Poll interval while waiting for the ready probe.
const READY_POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Number of startup attempts before surfacing an error.
const STARTUP_ATTEMPTS: usize = 2;

/// Errors that can occur when starting the XCUITest Runner.
#[derive(Debug, thiserror::Error)]
pub enum RunnerStartError {
    #[error("XCUITest Runner project not found")]
    /// The runner Xcode project could not be located.
    ProjectNotFound,
    #[error("No booted simulator found: {0}")]
    /// No booted simulator was available for starting the runner.
    NoBootedSimulator(String),
    #[error("Build failed: {0}")]
    /// Building the runner app bundles failed.
    BuildFailed(String),
    #[error("Build products not found (searched: {0})")]
    /// Expected runner app bundles could not be found after searching known paths.
    BuildProductsNotFound(String),
    #[error(
        "Failed to start XCUITest Runner for simulator {udid}: {details}. See log: {log_path}"
    )]
    /// Launching or supervising the xcodebuild test process failed.
    StartFailed {
        udid: String,
        details: String,
        log_path: String,
    },
    #[error(
        "XCUITest Runner was not ready for simulator {udid} after {timeout_secs} seconds. See log: {log_path}"
    )]
    /// The runner did not pass the `/ready` probe before the timeout elapsed.
    ReadyCheckTimeout {
        udid: String,
        timeout_secs: u64,
        log_path: String,
    },
    #[error("App install failed: {0}")]
    /// Installing one of the runner app bundles failed.
    InstallFailed(String),
    #[error("App launch failed: {0}")]
    /// Launching the runner app failed.
    LaunchFailed(String),
    #[error("Health check timed out after {0} seconds")]
    /// The runner did not become healthy before the timeout elapsed.
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
pub struct XCUITestRunner {
    /// UDID of the target simulator
    udid: String,
    /// Port for the HTTP server
    port: u16,
}

impl XCUITestRunner {
    /// Create a runner handle bound to the given simulator UDID and port.
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
        cleanup_runner_apps(&self.udid);
        Ok(())
    }

    /// Get a client connected to this runner.
    pub fn client(&self) -> XCUITestClient {
        XCUITestClient::new(self.port)
    }
}

impl Drop for XCUITestRunner {
    fn drop(&mut self) {
        cleanup_runner_apps(&self.udid);
    }
}

fn cleanup_runner_apps(udid: &str) {
    let _ = crate::coresim::terminate_app(udid, RUNNER_XCTRUNNER_BUNDLE_ID);
    let _ = crate::coresim::terminate_app(udid, RUNNER_HOST_BUNDLE_ID);
    cleanup_runner_processes(udid);
}

fn runner_process_pattern(udid: &str) -> String {
    format!(
        "XCUITestRunner.xcodeproj.*id={}.*{}",
        udid, RUNNER_TEST_ENTRYPOINT
    )
}

fn cleanup_runner_processes(udid: &str) {
    let pattern = runner_process_pattern(udid);

    let _ = std::process::Command::new("pkill")
        .args(["-TERM", "-f", &pattern])
        .output();

    std::thread::sleep(Duration::from_millis(500));

    let still_running = std::process::Command::new("pgrep")
        .args(["-f", &pattern])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if still_running {
        let _ = std::process::Command::new("pkill")
            .args(["-KILL", "-f", &pattern])
            .output();
    }
}

fn runner_log_path(udid: &str) -> PathBuf {
    std::env::temp_dir().join(format!("agent-mobile-xcuitest-runner-{}.log", udid))
}

fn resolve_target_device(
    target_udid: Option<&str>,
) -> Result<crate::coresim::BootedDevice, RunnerStartError> {
    match target_udid {
        Some(udid) => {
            let simulators = crate::simctl::list_simulators()
                .map_err(|e| RunnerStartError::NoBootedSimulator(e.to_string()))?;
            let device = simulators
                .into_iter()
                .find(|device| device.udid == udid)
                .ok_or_else(|| {
                    RunnerStartError::NoBootedSimulator(format!(
                        "Simulator with UDID {} was not found",
                        udid
                    ))
                })?;

            if device.state.as_deref() != Some("Booted") {
                return Err(RunnerStartError::NoBootedSimulator(format!(
                    "Simulator {} is not booted",
                    udid
                )));
            }

            Ok(crate::coresim::BootedDevice {
                udid: device.udid,
                name: device.name,
            })
        }
        None => {
            let device = crate::simctl::get_booted_simulator()
                .map_err(|e| RunnerStartError::NoBootedSimulator(e.to_string()))?;
            Ok(crate::coresim::BootedDevice {
                udid: device.udid,
                name: device.name,
            })
        }
    }
}

async fn ensure_build_products(
    project_path: &Path,
    udid: &str,
    force_build: bool,
) -> Result<(), RunnerStartError> {
    if force_build || RunnerBuildProducts::find().is_none() {
        eprintln!("Building XCUITest Runner for simulator {udid}...");
        build_for_testing(project_path, udid).await?;
    }

    if RunnerBuildProducts::find().is_none() {
        return Err(RunnerStartError::BuildProductsNotFound(
            RunnerBuildProducts::searched_paths_description(),
        ));
    }

    Ok(())
}

async fn stop_existing_runner(client: &XCUITestClient, fallback_udid: &str) {
    if let Ok(health) = client.health_status().await {
        if let Some(udid) = health.udid.as_deref() {
            cleanup_runner_apps(udid);
            if udid != fallback_udid {
                cleanup_runner_apps(fallback_udid);
            }
            return;
        }
    }

    cleanup_runner_apps(fallback_udid);
}

fn start_failed(udid: &str, log_path: &Path, details: impl Into<String>) -> RunnerStartError {
    RunnerStartError::StartFailed {
        udid: udid.to_string(),
        details: details.into(),
        log_path: log_path.display().to_string(),
    }
}

/// Start the XCUITest Runner via `xcodebuild test-without-building`.
pub async fn start_runner_detached(
    project_path: &Path,
    udid: &str,
    log_path: &Path,
) -> Result<Child, RunnerStartError> {
    let project_str = project_path
        .to_str()
        .ok_or(RunnerStartError::ProjectNotFound)?;

    let mut log_file = File::create(log_path)
        .map_err(|e| start_failed(udid, log_path, format!("failed to create log file: {}", e)))?;
    writeln!(
        log_file,
        "xcodebuild test-without-building -project {} -scheme XCUITestRunner -destination id={} -only-testing {}",
        project_str, udid, RUNNER_TEST_ENTRYPOINT
    )
    .map_err(|e| start_failed(udid, log_path, format!("failed to write log header: {}", e)))?;

    let stdout = log_file
        .try_clone()
        .map_err(|e| start_failed(udid, log_path, format!("failed to clone log file: {}", e)))?;

    let mut command = Command::new("xcodebuild");
    command.kill_on_drop(false);
    command.args([
        "test-without-building",
        "-project",
        project_str,
        "-scheme",
        "XCUITestRunner",
        "-destination",
        &format!("id={}", udid),
        "-only-testing",
        RUNNER_TEST_ENTRYPOINT,
    ]);
    command.stdout(Stdio::from(stdout));
    command.stderr(Stdio::from(log_file));

    let child = command
        .spawn()
        .map_err(|e| start_failed(udid, log_path, format!("failed to spawn xcodebuild: {}", e)))?;

    Ok(child)
}

async fn wait_for_runner_ready(
    child: &mut Child,
    port: u16,
    udid: &str,
    log_path: &Path,
) -> Result<(), RunnerStartError> {
    let client = XCUITestClient::new(port);
    let start = Instant::now();
    let mut last_progress = Instant::now();

    loop {
        if let Ok(ready) = client.ready_status().await {
            if ready.status == "ready" && ready.udid.as_deref() == Some(udid) {
                return Ok(());
            }
        }

        if let Some(status) = child.try_wait().map_err(|e| {
            start_failed(udid, log_path, format!("failed to poll xcodebuild: {}", e))
        })? {
            return Err(start_failed(
                udid,
                log_path,
                format!(
                    "xcodebuild exited before /ready succeeded (status: {})",
                    status
                ),
            ));
        }

        if start.elapsed() >= READY_TIMEOUT {
            return Err(RunnerStartError::ReadyCheckTimeout {
                udid: udid.to_string(),
                timeout_secs: READY_TIMEOUT.as_secs(),
                log_path: log_path.display().to_string(),
            });
        }

        if last_progress.elapsed() >= Duration::from_secs(5) {
            eprintln!(
                "Still waiting for XCUITest Runner on simulator {} ({}s elapsed)...",
                udid,
                start.elapsed().as_secs()
            );
            last_progress = Instant::now();
        }

        tokio::time::sleep(READY_POLL_INTERVAL).await;
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

/// Ensure the XCUITest Runner is started on the intended simulator and ready
/// to accept XCUITest-backed requests.
pub async fn ensure_runner_started(
    port: u16,
    target_udid: Option<&str>,
) -> Result<String, RunnerStartError> {
    let target = resolve_target_device(target_udid)?;
    let client = XCUITestClient::new(port);

    if let Ok(ready) = client.ready_status().await {
        if ready.status == "ready" && ready.udid.as_deref() == Some(target.udid.as_str()) {
            return Ok(target.udid);
        }
    }

    if let Ok(health) = client.health_status().await {
        if health.udid.as_deref() != Some(target.udid.as_str()) {
            eprintln!(
                "Stopping stale XCUITest Runner before switching to simulator {} ({})...",
                target.name, target.udid
            );
        } else {
            eprintln!(
                "Restarting stale XCUITest Runner on simulator {} ({})...",
                target.name, target.udid
            );
        }
        stop_existing_runner(&client, &target.udid).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let project_path =
        XCUITestRunner::bundled_project_path().ok_or(RunnerStartError::ProjectNotFound)?;

    for attempt in 1..=STARTUP_ATTEMPTS {
        let force_build = attempt > 1;
        ensure_build_products(&project_path, &target.udid, force_build).await?;

        cleanup_runner_apps(&target.udid);
        tokio::time::sleep(Duration::from_millis(500)).await;

        let log_path = runner_log_path(&target.udid);
        eprintln!(
            "Starting XCUITest Runner on simulator {} ({}) [attempt {}/{}]...",
            target.name, target.udid, attempt, STARTUP_ATTEMPTS
        );
        let mut child = start_runner_detached(&project_path, &target.udid, &log_path).await?;

        match wait_for_runner_ready(&mut child, port, &target.udid, &log_path).await {
            Ok(()) => {
                info!(simulator = %target.name, udid = %target.udid, "XCUITest Runner started successfully");
                return Ok(target.udid.clone());
            }
            Err(error) if attempt < STARTUP_ATTEMPTS => {
                eprintln!(
                    "XCUITest Runner was not ready on simulator {} ({}). Cleaning up and retrying once...",
                    target.name, target.udid
                );
                let _ = child.start_kill();
                let _ = child.wait().await;
                cleanup_runner_apps(&target.udid);
                tokio::time::sleep(Duration::from_millis(500)).await;
                let _ = error;
            }
            Err(error) => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                cleanup_runner_apps(&target.udid);
                return Err(error);
            }
        }
    }

    unreachable!("startup attempts are bounded and always return")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_log_path_contains_udid() {
        let path = runner_log_path("ABC-123");
        assert!(path.to_string_lossy().contains("ABC-123"));
    }

    #[test]
    fn test_searched_paths_description_mentions_derived_data() {
        let description = RunnerBuildProducts::searched_paths_description();
        assert!(description.contains("DerivedData"));
    }

    #[test]
    fn test_runner_process_pattern_targets_udid_and_entrypoint() {
        let pattern = runner_process_pattern("ABC-123");
        assert!(pattern.contains("ABC-123"));
        assert!(pattern.contains("testStartAutomationServer"));
    }
}
