//! Android device API

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
        Ok(Self {
            udid: udid.map(|s| s.to_string()),
        })
    }

    /// Stream logs from the device via adb logcat
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
