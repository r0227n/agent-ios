//! Console streaming for iOS and Android devices

use agent_mobile_core::{OutputWriter, Platform};
use std::io::Write;
use tokio::sync::watch;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Stream console logs from a device
///
/// This function handles both iOS and Android console streaming.
/// Press Ctrl+C to stop streaming (handled by stop_rx).
pub async fn stream_console_logs(
    platform: Platform,
    udid: Option<&str>,
    writer: OutputWriter,
    stop_rx: watch::Receiver<bool>,
) -> Result<()> {
    match platform {
        Platform::Ios => stream_ios_console(udid, writer, stop_rx).await,
        Platform::Android => stream_android_console(udid, writer, stop_rx).await,
    }
}

/// Stream iOS console logs using simctl
async fn stream_ios_console(
    udid: Option<&str>,
    mut writer: OutputWriter,
    mut stop_rx: watch::Receiver<bool>,
) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    let device_udid = if let Some(id) = udid {
        id.to_string()
    } else {
        // Auto-select booted device
        "booted".to_string()
    };

    // Start log stream using simctl
    let mut child = Command::new("xcrun")
        .args([
            "simctl",
            "spawn",
            &device_udid,
            "log",
            "stream",
            "--style",
            "compact",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    loop {
        tokio::select! {
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() {
                    child.kill().await.ok();
                    let _ = child.wait().await;
                    break;
                }
            }
            line = reader.next_line() => {
                match line? {
                    Some(line) => {
                        writeln!(writer, "{}", line)?;
                        writer.flush()?;
                    }
                    None => break,
                }
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

/// Stream Android console logs
async fn stream_android_console(
    udid: Option<&str>,
    mut writer: OutputWriter,
    mut stop_rx: watch::Receiver<bool>,
) -> Result<()> {
    use crate::AndroidDevice;

    // Connect via AndroidDevice
    let device = AndroidDevice::connect(udid).await?;

    // Stream logs
    let mut stream = device.stream_logs().await?;

    loop {
        tokio::select! {
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() {
                    stream.stop().await?;
                    break;
                }
            }
            line = stream.next_line() => {
                match line? {
                    Some(line) => {
                        writeln!(writer, "{}", line)?;
                        writer.flush()?;
                    }
                    None => break,
                }
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}
