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

/// Stream iOS console logs
async fn stream_ios_console(
    udid: Option<&str>,
    mut writer: OutputWriter,
    mut stop_rx: watch::Receiver<bool>,
) -> Result<()> {
    use crate::IosDevice;
    use agent_mobile_platform_ios::proto::idb::log_request::Source as LogSource;

    // Connect via IosDevice
    let mut device = IosDevice::connect(udid).await?;

    // Stream logs
    let mut response_stream: tonic::Streaming<agent_mobile_platform_ios::proto::idb::LogResponse> =
        device.stream_logs(LogSource::Target, vec![]).await?;

    loop {
        tokio::select! {
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() {
                    break;
                }
            }
            response = response_stream.message() => {
                match response? {
                    Some(log_response) => {
                        writer.write_all(&log_response.output)?;
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
    let mut device = AndroidDevice::connect(udid).await?;

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
