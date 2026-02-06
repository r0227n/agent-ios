//! Android device API
//!
//! Uses the native ADB protocol via `agent_mobile_platform_android`.

pub use agent_mobile_platform_android::LogcatStream;

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

    /// Stream logs from the device via ADB native protocol.
    pub async fn stream_logs(&self) -> Result<LogcatStream> {
        LogcatStream::open(self.udid.as_deref())
            .await
            .map_err(|e| e.into())
    }

    /// Get the device UDID
    pub fn udid(&self) -> Option<&str> {
        self.udid.as_deref()
    }
}
