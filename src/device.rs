//! Device command implementation.
//!
//! Provides cross-platform device management including list, boot,
//! shutdown, and other device lifecycle operations.

use clap::{Args, Subcommand};
use serde::Serialize;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::CommandResult;
use crate::helpers::format::OutputFormat;

/// Device command arguments.
#[derive(Args, Debug)]
pub struct DeviceArgs {
    /// Optional device subcommand. If omitted, all devices are listed.
    #[command(subcommand)]
    pub command: Option<DeviceCommands>,

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
    },

    /// Copy text to device clipboard (pbcopy).
    Pbcopy {
        /// Text to copy to clipboard.
        text: String,

        /// Device UDID/serial (optional, auto-detected if not specified).
        #[arg(short = 'u', long)]
        udid: Option<String>,
    },

    /// Paste from device clipboard (pbpaste).
    Pbpaste {
        /// Device UDID/serial (optional, auto-detected if not specified).
        #[arg(short = 'u', long)]
        udid: Option<String>,
    },
}

/// Unified device info for output.
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    /// Human-readable device name.
    pub name: String,
    /// Simulator UDID or Android device serial.
    pub udid: String,
    /// Platform identifier such as `ios` or `android`.
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Current runtime state, when available.
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Operating system version, when available.
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Platform-specific device classification, when available.
    pub device_type: Option<String>,
}

/// Detect platform from UDID by searching device lists.
pub async fn detect_platform_from_udid(
    udid: &str,
) -> Result<Platform, Box<dyn std::error::Error + Send + Sync>> {
    // Check iOS devices first
    let mut last_err: Option<String> = None;

    match agent_mobile_platform_ios::simctl::list_simulators() {
        Ok(targets) => {
            if targets.iter().any(|t| t.udid == udid) {
                return Ok(Platform::Ios);
            }
        }
        Err(e) => last_err = Some(format!("Failed to list iOS devices: {e}")),
    }

    // Check Android devices
    if agent_mobile_platform_android::adb::is_adb_available() {
        match agent_mobile_platform_android::adb::list_devices() {
            Ok(devices) => {
                if devices.iter().any(|(serial, _)| serial == udid) {
                    return Ok(Platform::Android);
                }
            }
            Err(e) => last_err = Some(format!("Failed to list Android devices: {e}")),
        }
    }

    if let Some(err) = last_err {
        return Err(format!("Platform detection failed for '{}': {}", udid, err).into());
    }
    Err(format!("Device not found: {}", udid).into())
}

/// Execute the device command.
pub async fn run(args: DeviceArgs) -> CommandResult {
    match args.command {
        Some(DeviceCommands::List { platform, format }) => {
            let platform_filter = match platform.as_deref() {
                Some(p) => Some(
                    p.parse::<Platform>()
                        .map_err(|e: String| Box::<dyn std::error::Error + Send + Sync>::from(e))?,
                ),
                None => None,
            };
            execute_list(platform_filter, &format).await
        }
        Some(DeviceCommands::Boot {
            name,
            platform,
            headless,
        }) => {
            let platform = match platform.as_deref() {
                Some(p) => DeviceResolver::parse_platform(p)?,
                None => DeviceResolver::detect_platform().await?,
            };
            execute_boot(platform, &name, headless).await
        }
        Some(DeviceCommands::Shutdown { udid }) => {
            let platform = detect_platform_from_udid(&udid).await?;
            execute_shutdown(platform, &udid).await
        }
        Some(DeviceCommands::Pbcopy { text, udid }) => {
            let (platform, resolved_udid) = match &udid {
                Some(u) => (detect_platform_from_udid(u).await?, u.clone()),
                None => {
                    let platform = DeviceResolver::detect_platform().await?;
                    let udid = get_default_udid(platform).await?;
                    (platform, udid)
                }
            };
            execute_pbcopy(platform, &resolved_udid, &text).await
        }
        Some(DeviceCommands::Pbpaste { udid }) => {
            let (platform, resolved_udid) = match &udid {
                Some(u) => (detect_platform_from_udid(u).await?, u.clone()),
                None => {
                    let platform = DeviceResolver::detect_platform().await?;
                    let udid = get_default_udid(platform).await?;
                    (platform, udid)
                }
            };
            execute_pbpaste(platform, &resolved_udid).await
        }
        None => {
            // Default: list both platforms
            execute_list(None, &args.format).await
        }
    }
}

/// Execute device list.
/// If platform_filter is None, lists both iOS and Android devices.
async fn execute_list(platform_filter: Option<Platform>, format: &OutputFormat) -> CommandResult {
    use agent_mobile_platform_android::adb;
    use std::collections::HashSet;

    let mut all_devices: Vec<DeviceInfo> = Vec::new();

    // iOS devices (if no filter or ios filter)
    if platform_filter.is_none() || platform_filter == Some(Platform::Ios) {
        let targets = agent_mobile_platform_ios::simctl::list_simulators().unwrap_or_default();
        for t in targets {
            all_devices.push(DeviceInfo {
                name: t.name.clone(),
                udid: t.udid.clone(),
                platform: "ios".to_string(),
                state: t.state.clone(),
                os_version: t.os_version.clone(),
                device_type: Some(t.target_type.as_str().to_string()),
            });
        }
    }

    // Android devices (if no filter or android filter)
    if platform_filter.is_none() || platform_filter == Some(Platform::Android) {
        let connected = adb::list_devices().unwrap_or_default();
        let mut running_avd_names: HashSet<String> = HashSet::new();

        for (serial, state) in &connected {
            let model = adb::get_device_model(serial).ok().flatten();
            let version = adb::get_android_version(serial).ok().flatten();
            let avd_name = adb::get_avd_name(serial).ok().flatten();

            if let Some(ref name) = avd_name {
                running_avd_names.insert(name.clone());
            }

            let name = model
                .clone()
                .or(avd_name.clone())
                .unwrap_or_else(|| serial.clone());

            all_devices.push(DeviceInfo {
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

        // Add available AVDs (not running)
        if let Ok(avds) = adb::list_avds() {
            for avd_name in avds {
                let normalized_name = avd_name.replace("_", " ");
                if !running_avd_names.contains(&normalized_name)
                    && !running_avd_names.contains(&avd_name)
                {
                    all_devices.push(DeviceInfo {
                        name: avd_name.clone(),
                        udid: avd_name,
                        platform: "android".to_string(),
                        state: Some("Shutdown".to_string()),
                        os_version: None,
                        device_type: Some("emulator".to_string()),
                    });
                }
            }
        }
    }

    // Output
    if format.is_json() {
        println!("{}", serde_json::to_string_pretty(&all_devices)?);
    } else {
        let platform_label = match platform_filter {
            Some(Platform::Ios) => "iOS Devices/Simulators",
            Some(Platform::Android) => "Android Devices/Emulators",
            None => "All Devices",
        };
        println!("{} ({}):", platform_label, all_devices.len());
        println!("{:-<80}", "");
        for d in &all_devices {
            let state = d.state.as_deref().unwrap_or("Unknown");
            let os = d.os_version.as_deref().unwrap_or("Unknown");
            let os_display = if d.platform == "android" {
                format!("Android {}", os)
            } else {
                os.to_string()
            };
            println!(
                "{:<30} | {} | {:<10} | {}",
                d.name, d.udid, state, os_display
            );
        }
    }
    Ok(())
}

/// Execute device boot.
async fn execute_boot(platform: Platform, name: &str, headless: bool) -> CommandResult {
    match platform {
        Platform::Ios => {
            use agent_mobile_platform_ios::simctl::management;

            // Check if name looks like a UDID or a device name
            let udid = if name.contains('-') && name.len() > 30 {
                // Looks like a UDID
                name.to_string()
            } else {
                // Try to find by name
                let targets =
                    agent_mobile_platform_ios::simctl::list_simulators().unwrap_or_default();
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
            use agent_mobile_platform_ios::simctl::management;

            management::shutdown(udid)?;
            println!("Shutdown simulator: {}", udid);
            Ok(())
        }
        Platform::Android => {
            use agent_mobile_platform_android::AdbConnection;

            let mut conn = AdbConnection::new(udid)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            conn.emu_kill()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

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
        Platform::Ios => Ok(agent_mobile_platform_ios::simctl::get_booted_simulator()?.udid),
        Platform::Android => {
            let devices = agent_mobile_platform_android::adb::list_devices()?;
            if let Some((serial, _)) = devices.first() {
                Ok(serial.clone())
            } else {
                Err("No Android device connected".into())
            }
        }
    }
}

/// Execute copy to clipboard.
async fn execute_pbcopy(platform: Platform, _udid: &str, text: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::helpers::client::with_xcuitest;

            let text = text.to_string();
            with_xcuitest(Some(_udid), |client| async move {
                client.clipboard_copy(&text).await?;
                println!("Copied to clipboard");
                Ok(())
            })
            .await
        }
        Platform::Android => Err("Android clipboard is not supported via adb".into()),
    }
}

/// Execute paste from clipboard.
async fn execute_pbpaste(platform: Platform, _udid: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::helpers::client::with_xcuitest;

            with_xcuitest(Some(_udid), |client| async move {
                let text = client.clipboard_paste().await?;
                print!("{}", text);
                Ok(())
            })
            .await
        }
        Platform::Android => Err("Android clipboard is not supported via adb".into()),
    }
}
