//! screenshot コマンド - スクリーンショット
//!
//! ```bash
//! agent-mobile screenshot output.png
//! agent-mobile screenshot              # base64 出力
//! agent-mobile screenshot -            # stdout に PNG バイナリ
//! ```

use std::io::Write;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use clap::Args;

use crate::cli::helpers::{CommandResult, DeviceArgs};
use crate::platform::ios::simctl::management as simctl;

use super::tap::resolve_platform;
use crate::types::Platform;

/// screenshot コマンド引数
#[derive(Args, Debug)]
pub struct ScreenshotArgs {
    /// Output path (omit for base64, "-" for stdout binary)
    pub path: Option<String>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the screenshot command
pub async fn run(args: ScreenshotArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;

    match platform {
        Platform::Ios => {
            execute_screenshot_ios(args.device.udid.as_deref(), args.path.as_deref()).await
        }
        Platform::Android => {
            execute_screenshot_android(args.device.udid.as_deref(), args.path.as_deref()).await
        }
    }
}

/// Execute screenshot on iOS using xcrun simctl
async fn execute_screenshot_ios(udid: Option<&str>, path: Option<&str>) -> CommandResult {
    // Determine UDID
    let udid = match udid {
        Some(u) => u.to_string(),
        None => get_booted_simulator_udid().await?,
    };

    // Handle output based on path
    match path {
        Some("-") => {
            // Output raw binary to stdout
            let image_data = simctl::io_screenshot_bytes(&udid)?;
            let mut stdout = std::io::stdout();
            stdout.write_all(&image_data)?;
            stdout.flush()?;
        }
        Some(p) => {
            // Write to file
            simctl::io_screenshot(&udid, p)?;
        }
        None => {
            // Output base64
            let image_data = simctl::io_screenshot_bytes(&udid)?;
            let base64 = BASE64.encode(&image_data);
            println!("{}", base64);
        }
    }

    Ok(())
}

/// Get the UDID of a booted simulator
async fn get_booted_simulator_udid() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::process::Command;

    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "booted", "-j"])
        .output()
        .await?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    json["devices"]
        .as_object()
        .and_then(|devices| {
            devices.values().find_map(|sims| {
                sims.as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|sim| sim["udid"].as_str())
                    .map(|s| s.to_string())
            })
        })
        .ok_or_else(|| "No booted simulator found".into())
}

/// Execute screenshot on Android
async fn execute_screenshot_android(udid: Option<&str>, path: Option<&str>) -> CommandResult {
    use tokio::process::Command;

    // Use adb to capture screenshot
    let serial_arg = udid
        .map(|s| vec!["-s".to_string(), s.to_string()])
        .unwrap_or_default();

    let output = Command::new("adb")
        .args(&serial_arg)
        .args(["exec-out", "screencap", "-p"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("adb screencap failed: {}", stderr).into());
    }

    let image_data = output.stdout;

    match path {
        Some("-") => {
            // Output raw binary to stdout
            let mut stdout = std::io::stdout();
            stdout.write_all(&image_data)?;
            stdout.flush()?;
        }
        Some(p) => {
            // Write to file
            std::fs::write(p, &image_data)?;
        }
        None => {
            // Output base64
            let base64 = BASE64.encode(&image_data);
            println!("{}", base64);
        }
    }

    Ok(())
}
