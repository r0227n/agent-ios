//! screenshot コマンド - スクリーンショット
//!
//! ```bash
//! agent-mobile screenshot output.png
//! agent-mobile screenshot              # base64 出力
//! agent-mobile screenshot -            # stdout に PNG バイナリ
//! ```

use std::io::Write;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use clap::Args;
use tokio::time::sleep;

use crate::cli::helpers::{with_client, CommandResult, DeviceArgs, OutputWriter};

use super::tap::resolve_platform;
use crate::types::Platform;

/// Maximum retry attempts
const MAX_RETRIES: u32 = 5;

/// Initial retry delay
const INITIAL_RETRY_DELAY_MS: u64 = 1000;

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

/// Execute screenshot on iOS with retry logic and simctl fallback
async fn execute_screenshot_ios(udid: Option<&str>, path: Option<&str>) -> CommandResult {
    let mut attempt = 0;

    loop {
        attempt += 1;

        match try_screenshot_ios(udid, path).await {
            Ok(()) => return Ok(()),
            Err(e) if should_retry(e.as_ref(), attempt) => {
                let delay_ms = INITIAL_RETRY_DELAY_MS * (1 << (attempt - 1));
                eprintln!(
                    "Screenshot attempt {} failed (framebuffer not ready), retrying in {}ms...",
                    attempt, delay_ms
                );
                sleep(Duration::from_millis(delay_ms)).await;
            }
            Err(e) => {
                if is_framebuffer_error(e.as_ref()) && attempt >= MAX_RETRIES {
                    // Fallback to xcrun simctl for simulators
                    eprintln!("Falling back to xcrun simctl...");
                    return try_screenshot_simctl(udid, path).await;
                }
                return Err(e);
            }
        }
    }
}

/// Try to take screenshot on iOS
async fn try_screenshot_ios(udid: Option<&str>, path: Option<&str>) -> CommandResult {
    let path = path.map(|s| s.to_string());

    with_client(udid, |mut client| async move {
        let image_data = client.screenshot().await?;

        match path.as_deref() {
            Some("-") => {
                // Output raw binary to stdout
                let mut stdout = std::io::stdout();
                stdout.write_all(&image_data)?;
                stdout.flush()?;
            }
            Some(p) => {
                // Write to file
                let mut writer = OutputWriter::from_path(p)?;
                writer.write_all(&image_data)?;
                writer.flush()?;
            }
            None => {
                // Output base64
                let base64 = BASE64.encode(&image_data);
                println!("{}", base64);
            }
        }

        Ok(())
    })
    .await
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

/// Check if error is retryable
fn should_retry(error: &dyn std::error::Error, attempt: u32) -> bool {
    if attempt >= MAX_RETRIES {
        return false;
    }
    is_framebuffer_error(error)
}

/// Check if error is framebuffer related
fn is_framebuffer_error(error: &dyn std::error::Error) -> bool {
    error.to_string().contains("No Image available to encode")
}

/// Fallback screenshot using xcrun simctl (for iOS Simulators)
async fn try_screenshot_simctl(udid: Option<&str>, path: Option<&str>) -> CommandResult {
    use tokio::process::Command;

    // Determine UDID
    let udid = match udid {
        Some(u) => u.to_string(),
        None => {
            // Get booted simulator UDID
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
                .ok_or_else(|| "No booted simulator found")?
        }
    };

    // For base64 or stdout output, use a temp file
    let (temp_path, is_temp) = match path {
        Some("-") | None => {
            let temp = format!("/tmp/agent_mobile_screenshot_{}.png", std::process::id());
            (temp, true)
        }
        Some(p) => (p.to_string(), false),
    };

    // Execute xcrun simctl io screenshot
    let output = Command::new("xcrun")
        .args(["simctl", "io", &udid, "screenshot", &temp_path])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("xcrun simctl screenshot failed: {}", stderr).into());
    }

    // Handle output based on original request
    if is_temp {
        let image_data = std::fs::read(&temp_path)?;
        std::fs::remove_file(&temp_path).ok(); // Clean up temp file

        match path {
            Some("-") => {
                let mut stdout = std::io::stdout();
                stdout.write_all(&image_data)?;
                stdout.flush()?;
            }
            None => {
                let base64 = BASE64.encode(&image_data);
                println!("{}", base64);
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
