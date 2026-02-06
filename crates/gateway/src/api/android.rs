//! Android device API
//!
//! Uses the native ADB protocol via `agent_mobile_platform_android::AdbConnection`.

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// High-level Android device API
pub struct AndroidDevice {
    udid: Option<String>,
}

impl AndroidDevice {
    /// Connect to an Android device
    ///
    /// If `udid` is provided, connects to that specific device.
    /// Otherwise, auto-selects an available device.
    pub async fn connect(udid: Option<&str>) -> Result<Self> {
        // Verify ADB server is reachable (use spawn_blocking to avoid blocking the runtime)
        let available =
            tokio::task::spawn_blocking(agent_mobile_platform_android::is_adb_available)
                .await
                .map_err(|e| format!("Failed to check ADB availability: {}", e))?;
        if !available {
            return Err(
                "ADB server not reachable at 127.0.0.1:5037. Please start the ADB server with 'adb start-server'.".into()
            );
        }
        Ok(Self {
            udid: udid.map(|s| s.to_string()),
        })
    }

    /// Stream logs from the device.
    ///
    /// Note: logcat streaming requires a persistent process for continuous output.
    /// We use `adb logcat` process for this since ADB protocol's logcat
    /// is a blocking operation that doesn't work well with async streaming.
    /// This is the one remaining `adb` CLI usage, kept for its streaming nature.
    pub async fn stream_logs(&mut self) -> Result<LogcatStream> {
        let mut cmd = Command::new("adb");
        if let Some(s) = &self.udid {
            cmd.args(["-s", s]);
        }
        cmd.arg("logcat")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = cmd.spawn().map_err(|e| {
            format!(
                "Failed to start adb logcat: {}. Make sure adb is installed.",
                e
            )
        })?;

        let stdout = child.stdout.take().ok_or("Failed to capture adb stdout")?;
        let reader = BufReader::new(stdout).lines();

        Ok(LogcatStream { child, reader })
    }

    /// Get the device UDID
    pub fn udid(&self) -> Option<&str> {
        self.udid.as_deref()
    }
}

/// Stream of logcat output
pub struct LogcatStream {
    child: Child,
    reader: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
}

impl LogcatStream {
    /// Read the next line from the logcat stream
    pub async fn next_line(&mut self) -> Result<Option<String>> {
        self.reader.next_line().await.map_err(Into::into)
    }

    /// Stop the logcat stream
    pub async fn stop(&mut self) -> Result<()> {
        self.child.kill().await?;
        self.child.wait().await?;
        Ok(())
    }
}

impl Drop for LogcatStream {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}
