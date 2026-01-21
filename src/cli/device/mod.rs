//! Device command implementation.
//!
//! Provides cross-platform device management including list, boot,
//! shutdown, and other device lifecycle operations.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::helpers::{CommandResult, OutputFormat};
use crate::types::Platform;

/// Device command arguments.
#[derive(Args, Debug)]
pub struct DeviceArgs {
    #[command(subcommand)]
    pub command: Option<DeviceCommands>,

    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Output format (text or json).
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

/// Device subcommands.
#[derive(Subcommand, Debug)]
pub enum DeviceCommands {
    /// List available devices/simulators.
    List {
        /// Platform (ios or android). Auto-detected if not specified.
        #[arg(short = 'p', long)]
        platform: Option<String>,

        /// Output format (text or json).
        #[arg(short = 'f', long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Boot a simulator/emulator by name or UDID.
    Boot {
        /// Simulator/emulator name or UDID to boot.
        name: String,

        /// Platform (ios or android). Auto-detected if not specified.
        #[arg(short = 'p', long)]
        platform: Option<String>,

        /// Boot in headless mode (no UI window).
        #[arg(long)]
        headless: bool,
    },

    /// Shutdown a simulator/emulator.
    Shutdown {
        /// Device UDID/serial to shutdown.
        #[arg(short, long)]
        udid: String,

        /// Platform (ios or android). Auto-detected if not specified.
        #[arg(short = 'p', long)]
        platform: Option<String>,
    },

    /// Copy text to device clipboard (pbcopy).
    Pbcopy {
        /// Text to copy to clipboard.
        text: String,

        /// Platform (ios or android). Auto-detected if not specified.
        #[arg(short = 'p', long)]
        platform: Option<String>,

        /// Device UDID/serial (optional, auto-detected if not specified).
        #[arg(short = 'u', long)]
        udid: Option<String>,
    },

    /// Paste from device clipboard (pbpaste).
    Pbpaste {
        /// Platform (ios or android). Auto-detected if not specified.
        #[arg(short = 'p', long)]
        platform: Option<String>,

        /// Device UDID/serial (optional, auto-detected if not specified).
        #[arg(short = 'u', long)]
        udid: Option<String>,
    },
}

/// Unified device info for output.
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub udid: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,
}

/// Detect platform based on available devices.
async fn detect_platform() -> Result<Platform, Box<dyn std::error::Error + Send + Sync>> {
    let ios_state_path = std::path::Path::new("/tmp/idb/state");
    if ios_state_path.exists() {
        return Ok(Platform::Ios);
    }

    if crate::platform::android::adb::is_adb_available() {
        let devices = crate::platform::android::adb::list_devices();
        if let Ok(devs) = devices {
            if !devs.is_empty() {
                return Ok(Platform::Android);
            }
        }
    }

    Ok(Platform::Ios)
}

/// Resolve platform from optional string.
async fn resolve_platform(
    platform_str: Option<&str>,
) -> Result<Platform, Box<dyn std::error::Error + Send + Sync>> {
    match platform_str {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() }),
        None => detect_platform().await,
    }
}

/// Execute the device command.
pub async fn run(args: DeviceArgs) -> CommandResult {
    match args.command {
        Some(DeviceCommands::List { platform, format }) => {
            let platform = resolve_platform(platform.as_deref()).await?;
            execute_list(platform, &format).await
        }
        Some(DeviceCommands::Boot {
            name,
            platform,
            headless,
        }) => {
            let platform = resolve_platform(platform.as_deref()).await?;
            execute_boot(platform, &name, headless).await
        }
        Some(DeviceCommands::Shutdown { udid, platform }) => {
            let platform = resolve_platform(platform.as_deref()).await?;
            execute_shutdown(platform, &udid).await
        }
        Some(DeviceCommands::Pbcopy {
            text,
            platform,
            udid,
        }) => {
            let platform = resolve_platform(platform.as_deref()).await?;
            let udid = match &udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_pbcopy(platform, &udid, &text).await
        }
        Some(DeviceCommands::Pbpaste { platform, udid }) => {
            let platform = resolve_platform(platform.as_deref()).await?;
            let udid = match &udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_pbpaste(platform, &udid).await
        }
        None => {
            // Default: list with top-level args
            let platform = resolve_platform(args.platform.as_deref()).await?;
            execute_list(platform, &args.format).await
        }
    }
}

/// Execute device list.
async fn execute_list(platform: Platform, format: &OutputFormat) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::companion::CompanionLister;

            let targets = match CompanionLister::new() {
                Ok(lister) => lister.list_targets(None).unwrap_or_default(),
                Err(_) => Vec::new(),
            };
            let devices: Vec<DeviceInfo> = targets
                .iter()
                .map(|t| DeviceInfo {
                    name: t.name.clone(),
                    udid: t.udid.clone(),
                    platform: "ios".to_string(),
                    state: t.state.clone(),
                    os_version: t.os_version.clone(),
                    device_type: Some(t.target_type.as_str().to_string()),
                })
                .collect();

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            } else {
                println!("iOS Devices/Simulators ({}):", devices.len());
                println!("{:-<70}", "");
                for d in &devices {
                    let state = d.state.as_deref().unwrap_or("Unknown");
                    let os = d.os_version.as_deref().unwrap_or("Unknown");
                    println!("{} | {} | {} | {}", d.name, d.udid, state, os);
                }
            }
            Ok(())
        }
        Platform::Android => {
            use crate::platform::android::adb;
            use std::collections::HashSet;

            // 1. 起動中デバイスを取得
            let connected = adb::list_devices().unwrap_or_default();
            let mut devices: Vec<DeviceInfo> = Vec::new();
            let mut running_avd_names: HashSet<String> = HashSet::new();

            for (serial, state) in &connected {
                let model = adb::get_device_model(serial).ok().flatten();
                let version = adb::get_android_version(serial).ok().flatten();
                let avd_name = adb::get_avd_name(serial).ok().flatten();

                // 起動中の AVD 名を記録
                if let Some(ref name) = avd_name {
                    running_avd_names.insert(name.clone());
                }

                let name = model
                    .clone()
                    .or(avd_name.clone())
                    .unwrap_or_else(|| serial.clone());

                devices.push(DeviceInfo {
                    name,
                    udid: serial.clone(),
                    platform: "android".to_string(),
                    state: Some(state.clone()),
                    os_version: version,
                    device_type: if adb::is_emulator(serial) {
                        Some("emulator".to_string())
                    } else {
                        Some("device".to_string())
                    },
                });
            }

            // 2. 利用可能な AVD を追加（未起動のもののみ）
            if let Ok(avds) = adb::list_avds() {
                for avd_name in avds {
                    let normalized_name = avd_name.replace("_", " ");
                    if !running_avd_names.contains(&normalized_name)
                        && !running_avd_names.contains(&avd_name)
                    {
                        devices.push(DeviceInfo {
                            name: avd_name.clone(),
                            udid: avd_name, // AVD 名を UDID として使用（boot 時に使用）
                            platform: "android".to_string(),
                            state: Some("Shutdown".to_string()),
                            os_version: None,
                            device_type: Some("emulator".to_string()),
                        });
                    }
                }
            }

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            } else {
                println!("Android Devices/Emulators ({}):", devices.len());
                println!("{:-<70}", "");
                for d in &devices {
                    let state = d.state.as_deref().unwrap_or("Unknown");
                    let os = d.os_version.as_deref().unwrap_or("Unknown");
                    println!("{} | {} | {} | Android {}", d.name, d.udid, state, os);
                }
            }
            Ok(())
        }
    }
}

/// Execute device boot.
async fn execute_boot(platform: Platform, name: &str, headless: bool) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::companion::CompanionLister;
            use crate::platform::ios::simctl::management;

            // Check if name looks like a UDID or a device name
            let udid = if name.contains('-') && name.len() > 30 {
                // Looks like a UDID
                name.to_string()
            } else {
                // Try to find by name
                let targets = match CompanionLister::new() {
                    Ok(lister) => lister.list_targets(None).unwrap_or_default(),
                    Err(_) => Vec::new(),
                };
                let matching = targets
                    .iter()
                    .find(|t| t.name.to_lowercase().contains(&name.to_lowercase()));

                match matching {
                    Some(t) => t.udid.clone(),
                    None => {
                        return Err(format!("No simulator found matching '{}'", name).into());
                    }
                }
            };

            management::boot(&udid)?;
            println!("Booted simulator: {}", udid);
            Ok(())
        }
        Platform::Android => {
            use tokio::process::Command;

            // For Android, use emulator command
            println!("Starting emulator: {}...", name);

            let mut cmd = Command::new("emulator");
            cmd.args(["-avd", name]);
            if headless {
                cmd.args(["-no-window", "-no-audio"]);
            }

            // Start in background
            cmd.spawn().map_err(|e: std::io::Error| {
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

            println!("Emulator '{}' starting...", name);
            Ok(())
        }
    }
}

/// Execute device shutdown.
async fn execute_shutdown(platform: Platform, udid: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::platform::ios::simctl::management;

            management::shutdown(udid)?;
            println!("Shutdown simulator: {}", udid);
            Ok(())
        }
        Platform::Android => {
            use tokio::process::Command;

            // For Android emulators, use adb emu kill
            let mut cmd = Command::new("adb");
            cmd.args(["-s", udid, "emu", "kill"]);

            let output = cmd.output().await.map_err(|e: std::io::Error| {
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to shutdown: {}", stderr).into());
            }

            println!("Shutdown emulator: {}", udid);
            Ok(())
        }
    }
}

/// Get default UDID for the platform.
async fn get_default_udid(
    platform: Platform,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match platform {
        Platform::Ios => {
            use crate::companion::CompanionLister;

            let targets = match CompanionLister::new() {
                Ok(lister) => lister.list_targets(None).unwrap_or_default(),
                Err(_) => Vec::new(),
            };

            // Find first booted simulator
            let booted = targets
                .iter()
                .find(|t| t.state.as_deref() == Some("Booted"));

            match booted {
                Some(t) => Ok(t.udid.clone()),
                None => Err("No booted iOS simulator found".into()),
            }
        }
        Platform::Android => {
            let devices = crate::platform::android::adb::list_devices()?;
            if let Some((serial, _)) = devices.first() {
                Ok(serial.clone())
            } else {
                Err("No Android device connected".into())
            }
        }
    }
}

/// Execute copy to clipboard.
async fn execute_pbcopy(platform: Platform, udid: &str, text: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::platform::ios::simctl::management;
            management::pbcopy(udid, text)?;
            println!("Copied to clipboard");
            Ok(())
        }
        Platform::Android => {
            // Android doesn't have a direct clipboard API via adb
            // We can use am broadcast but it's limited
            Err("Android clipboard is not supported via adb".into())
        }
    }
}

/// Execute paste from clipboard.
async fn execute_pbpaste(platform: Platform, udid: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::platform::ios::simctl::management;
            let text = management::pbpaste(udid)?;
            print!("{}", text);
            Ok(())
        }
        Platform::Android => Err("Android clipboard is not supported via adb".into()),
    }
}
