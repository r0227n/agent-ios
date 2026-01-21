//! Clipboard command implementation.
//!
//! Provides cross-platform clipboard operations (copy and paste).
//! iOS uses xcrun simctl pbcopy/pbpaste.

use clap::{Args, Subcommand};

use crate::cli::helpers::{CommandResult, DeviceArgs};
use crate::types::Platform;

/// Clipboard command arguments.
#[derive(Args, Debug)]
pub struct ClipboardArgs {
    #[command(subcommand)]
    pub command: ClipboardCommands,
}

/// Clipboard subcommands.
#[derive(Subcommand, Debug)]
pub enum ClipboardCommands {
    /// Copy text to clipboard.
    Copy {
        /// Text to copy to clipboard.
        text: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Paste from clipboard.
    Paste {
        #[command(flatten)]
        device: DeviceArgs,
    },
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

/// Execute the clipboard command.
pub async fn run(args: ClipboardArgs) -> CommandResult {
    match args.command {
        ClipboardCommands::Copy { text, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            let udid = match &device.udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_copy(platform, &udid, &text).await
        }
        ClipboardCommands::Paste { device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            let udid = match &device.udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_paste(platform, &udid).await
        }
    }
}

/// Execute copy to clipboard.
async fn execute_copy(platform: Platform, udid: &str, text: &str) -> CommandResult {
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
async fn execute_paste(platform: Platform, udid: &str) -> CommandResult {
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
