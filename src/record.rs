//! record コマンド - 画面録画
//!
//! iOS/Android両対応の画面録画をMP4形式で保存
//!
//! ```bash
//! agent-mobile record                        # Ctrl+Cまで録画、カレントディレクトリに保存
//! agent-mobile record --time-limit 30        # 30秒間録画
//! agent-mobile record --output /tmp/         # 出力ディレクトリ指定
//! agent-mobile record --output /tmp/demo.mp4 # 出力ファイル指定
//! agent-mobile record -p android -t 60       # Android 60秒録画
//! ```

use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::{setup_ctrl_c_handler, CommandResult, DeviceArgs};

/// record コマンド引数
#[derive(Args, Debug)]
pub struct RecordArgs {
    /// Output path (directory or file)
    ///
    /// Modes:
    /// - If path ends with '/': treat as directory and auto-generate filename
    /// - Otherwise: treat as full file path
    /// - If omitted: current directory with timestamp filename
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Recording time limit in seconds (records until Ctrl+C if not specified)
    #[arg(short = 't', long)]
    pub time_limit: Option<u64>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Generate timestamp-based filename for recording
fn generate_filename() -> String {
    let now = chrono::Local::now();
    format!("recording_{}.mp4", now.format("%Y%m%d_%H%M%S"))
}

/// Resolve output path based on user input
///
/// Logic:
/// 1. If `output` is None: current directory + timestamp filename
/// 2. If `output` ends with '/': directory + timestamp filename
/// 3. Otherwise: treat as full file path
fn resolve_output_path(output: Option<&str>) -> Result<String, std::io::Error> {
    match output {
        None => {
            // Default: current directory + timestamp filename
            Ok(generate_filename())
        }
        Some(out) => {
            // Check if it's a directory path
            if out.ends_with(std::path::MAIN_SEPARATOR) {
                // Directory specified, generate filename
                let filename = generate_filename();
                return Ok(format!("{}{}", out, filename));
            }

            // Check if path exists and is a directory
            let path_obj = std::path::Path::new(out);
            if path_obj.exists() && path_obj.is_dir() {
                let filename = generate_filename();
                return Ok(path_obj.join(filename).to_string_lossy().to_string());
            }

            // Treat as full file path
            Ok(out.to_string())
        }
    }
}

/// Execute the record command
pub async fn run(args: RecordArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;
    let resolved_path = resolve_output_path(args.output.as_deref())?;

    match platform {
        Platform::Ios => {
            execute_record_ios(args.device.udid.as_deref(), &resolved_path, args.time_limit).await
        }
        Platform::Android => {
            execute_record_android(args.device.udid.as_deref(), &resolved_path, args.time_limit)
                .await
        }
    }
}

/// Execute recording on iOS using xcrun simctl io recordVideo
async fn execute_record_ios(
    udid: Option<&str>,
    output: &str,
    time_limit: Option<u64>,
) -> CommandResult {
    use tokio::process::Command;

    // Determine UDID
    let udid = match udid {
        Some(u) => u.to_string(),
        None => get_booted_simulator_udid().await?,
    };

    if let Some(secs) = time_limit {
        eprintln!(
            "Recording video to {} (time limit: {} seconds, or press ^C to stop)",
            output, secs
        );
    } else {
        eprintln!("Recording video to {} until ^C", output);
    }

    // Start recording with xcrun simctl io recordVideo
    let mut record_cmd = Command::new("xcrun");
    record_cmd.args([
        "simctl",
        "io",
        &udid,
        "recordVideo",
        "--codec=h264",
        "--force",
        output,
    ]);

    let mut child = record_cmd
        .spawn()
        .map_err(|e| format!("Failed to start xcrun simctl io recordVideo: {}", e))?;

    // Setup Ctrl+C handler for early stop
    let stop_rx = setup_ctrl_c_handler();
    let mut stop_rx_clone = stop_rx.clone();

    // Wait for recording to complete, time limit, or Ctrl+C
    let final_status: CommandResult = if let Some(secs) = time_limit {
        tokio::select! {
            status = child.wait() => {
                match status {
                    Ok(s) if s.success() => {
                        eprintln!("\nRecording completed");
                        Ok(())
                    }
                    Ok(s) => {
                        Err(format!("recordVideo exited with status: {}", s).into())
                    }
                    Err(e) => {
                        Err(format!("Failed to wait for recordVideo: {}", e).into())
                    }
                }
            }
            _ = stop_rx_clone.changed() => {
                eprintln!("\nStopping recording...");
                send_sigint_to_child(&child);
                match child.wait().await {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(format!("recordVideo exited with status: {}", s).into()),
                    Err(e) => Err(format!("Failed to wait for recordVideo: {}", e).into()),
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(secs)) => {
                eprintln!("\nTime limit reached, stopping recording...");
                send_sigint_to_child(&child);
                match child.wait().await {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(format!("recordVideo exited with status: {}", s).into()),
                    Err(e) => Err(format!("Failed to wait for recordVideo: {}", e).into()),
                }
            }
        }
    } else {
        tokio::select! {
            status = child.wait() => {
                match status {
                    Ok(s) if s.success() => {
                        eprintln!("\nRecording completed");
                        Ok(())
                    }
                    Ok(s) => {
                        Err(format!("recordVideo exited with status: {}", s).into())
                    }
                    Err(e) => {
                        Err(format!("Failed to wait for recordVideo: {}", e).into())
                    }
                }
            }
            _ = stop_rx_clone.changed() => {
                eprintln!("\nStopping recording...");
                send_sigint_to_child(&child);
                match child.wait().await {
                    Ok(s) if s.success() => Ok(()),
                    Ok(s) => Err(format!("recordVideo exited with status: {}", s).into()),
                    Err(e) => Err(format!("Failed to wait for recordVideo: {}", e).into()),
                }
            }
        }
    };

    final_status?;
    println!("Video saved to: {}", output);
    Ok(())
}

/// Send SIGINT to a child process
fn send_sigint_to_child(child: &tokio::process::Child) {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        if let Some(pid) = child.id() {
            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGINT);
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!(
            "Warning: Graceful shutdown via SIGINT is not supported on this platform; child process (pid: {:?}) may continue running.",
            child.id()
        );
    }
}

/// Get the UDID of a booted simulator
async fn get_booted_simulator_udid() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::process::Command;

    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "booted", "-j"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("xcrun simctl list devices booted failed: {}", stderr).into());
    }

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

/// Execute recording on Android using adb screenrecord
async fn execute_record_android(
    udid: Option<&str>,
    output: &str,
    time_limit: Option<u64>,
) -> CommandResult {
    use tokio::process::Command;

    // Android screenrecord max is 180 seconds
    const ANDROID_MAX_RECORDING_SECS: u64 = 180;

    let effective_time_limit = time_limit.unwrap_or(ANDROID_MAX_RECORDING_SECS);
    if effective_time_limit > ANDROID_MAX_RECORDING_SECS {
        eprintln!(
            "Warning: Android screenrecord max time limit is {} seconds. Using that instead.",
            ANDROID_MAX_RECORDING_SECS
        );
    }
    let effective_time_limit = effective_time_limit.min(ANDROID_MAX_RECORDING_SECS);

    let remote_path = "/sdcard/screen_recording.mp4";

    // Build adb command arguments
    let serial_args: Vec<String> = udid
        .map(|s| vec!["-s".to_string(), s.to_string()])
        .unwrap_or_default();

    eprintln!(
        "Recording video to {} (time limit: {} seconds)",
        output, effective_time_limit
    );
    if time_limit.is_none() {
        eprintln!("Press Ctrl+C to stop recording early");
    }

    // Start recording
    let mut record_cmd = Command::new("adb");
    record_cmd.args(&serial_args);
    record_cmd.args([
        "shell",
        "screenrecord",
        "--time-limit",
        &effective_time_limit.to_string(),
        remote_path,
    ]);

    let mut child = record_cmd
        .spawn()
        .map_err(|e| format!("Failed to start adb screenrecord: {}", e))?;

    // Setup Ctrl+C handler for early stop
    let stop_rx = setup_ctrl_c_handler();
    let mut stop_rx_clone = stop_rx.clone();

    // Wait for recording to complete or Ctrl+C
    let recording_result: CommandResult = tokio::select! {
        status = child.wait() => {
            match status {
                Ok(s) if s.success() => {
                    eprintln!("\nRecording completed");
                    Ok(())
                }
                Ok(s) => {
                    Err(format!("screenrecord exited with status: {}", s).into())
                }
                Err(e) => {
                    Err(format!("Failed to wait for screenrecord: {}", e).into())
                }
            }
        }
        _ = stop_rx_clone.changed() => {
            eprintln!("\nStopping recording...");
            // Send SIGINT to the adb process to stop recording gracefully
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if let Some(pid) = child.id() {
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGINT);
                }
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill().await;
            }

            // Wait for process to finish and check status
            match child.wait().await {
                Ok(s) if s.success() => Ok(()),
                Ok(s) => Err(format!("screenrecord exited with status: {}", s).into()),
                Err(e) => Err(format!("Failed to wait for screenrecord: {}", e).into()),
            }
        }
    };

    // Only proceed to pull if recording succeeded
    recording_result?;

    // Pull the recorded file
    eprintln!("Pulling recorded video...");
    let mut pull_cmd = Command::new("adb");
    pull_cmd.args(&serial_args);
    pull_cmd.args(["pull", remote_path, output]);

    let pull_output = pull_cmd.output().await?;
    if !pull_output.status.success() {
        let stderr = String::from_utf8_lossy(&pull_output.stderr);
        return Err(format!("Failed to pull video file: {}", stderr).into());
    }

    // Cleanup remote file
    let mut rm_cmd = Command::new("adb");
    rm_cmd.args(&serial_args);
    rm_cmd.args(["shell", "rm", "-f", remote_path]);
    let _ = rm_cmd.output().await;

    println!("Video saved to: {}", output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_output_path_none() {
        let result = resolve_output_path(None);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with("recording_"));
        assert!(path.ends_with(".mp4"));
    }

    #[test]
    fn test_resolve_output_path_directory_with_slash() {
        let result = resolve_output_path(Some("/tmp/"));
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with("/tmp/recording_"));
        assert!(path.ends_with(".mp4"));
    }

    #[test]
    fn test_resolve_output_path_full_path() {
        let result = resolve_output_path(Some("/tmp/test.mp4"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/tmp/test.mp4");
    }

    #[test]
    fn test_generate_filename_format() {
        let filename = generate_filename();
        assert!(filename.starts_with("recording_"));
        assert!(filename.ends_with(".mp4"));
        assert!(filename.contains("_")); // timestamp separator
    }
}
