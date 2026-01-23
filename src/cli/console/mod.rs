//! Console command for streaming device logs
//!
//! This module provides the `console` command which streams real-time
//! console output from iOS and Android devices.

pub mod android;
pub mod ios;

use clap::Args;

use crate::cli::helpers::{CommandResult, DeviceArgs};

/// Console output streaming arguments
#[derive(Args)]
pub struct ConsoleArgs {
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Detected platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Ios,
    Android,
}

/// Run the console command
pub async fn run(args: ConsoleArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;

    match platform {
        Platform::Ios => ios::run(args.device.udid).await,
        Platform::Android => android::run(args.device.udid).await,
    }
}

/// Resolve platform from argument or auto-detect.
async fn resolve_platform(platform_arg: Option<&str>) -> CommandResult<Platform> {
    // If explicitly specified, use that
    if let Some(p) = platform_arg {
        return match p.to_lowercase().as_str() {
            "ios" => Ok(Platform::Ios),
            "android" => Ok(Platform::Android),
            _ => Err(format!("Unknown platform: {}. Use 'ios' or 'android'.", p).into()),
        };
    }

    // Auto-detect: check for iOS companion state first
    if has_ios_companion().await {
        return Ok(Platform::Ios);
    }

    // Then check for Android device
    if has_android_device().await {
        return Ok(Platform::Android);
    }

    Err(
        "No device found. Please connect an iOS simulator/device or Android emulator/device."
            .into(),
    )
}

/// Check if iOS companion is available.
async fn has_ios_companion() -> bool {
    use std::path::Path;

    // Check for companion state file
    let state_path = Path::new("/tmp/idb/state");
    if !state_path.exists() {
        return false;
    }

    // Try to read the state file and check if there are any companions
    if let Ok(content) = std::fs::read_to_string(state_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = json.as_array() {
                return !arr.is_empty();
            }
        }
    }

    false
}

/// Check if Android device is available via ADB.
async fn has_android_device() -> bool {
    use tokio::process::Command;

    let output = Command::new("adb").args(["devices", "-l"]).output().await;

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Check if there's at least one device listed (not just "List of devices attached")
            stdout.lines().skip(1).any(|line| {
                let line = line.trim();
                !line.is_empty() && (line.contains("device") || line.contains("emulator"))
            })
        }
        _ => false,
    }
}
