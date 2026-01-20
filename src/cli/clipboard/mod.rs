//! Clipboard command implementation.
//!
//! Provides cross-platform clipboard operations (copy and paste).
//! iOS uses xcrun simctl pbcopy/pbpaste.

use clap::Args;

use crate::cli::helpers::CommandResult;
use crate::types::Platform;

/// Clipboard command arguments.
#[derive(Args, Debug)]
pub struct ClipboardArgs {
    /// Copy text to clipboard.
    #[arg(long)]
    pub copy: Option<String>,

    /// Paste from clipboard.
    #[arg(long)]
    pub paste: bool,

    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,
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

/// Execute the clipboard command.
pub async fn run(args: ClipboardArgs) -> CommandResult {
    let platform = match &args.platform {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?,
        None => detect_platform().await?,
    };

    let udid = match &args.udid {
        Some(u) => u.clone(),
        None => get_default_udid(platform).await?,
    };

    if let Some(ref text) = args.copy {
        return execute_copy(platform, &udid, text).await;
    }

    if args.paste {
        return execute_paste(platform, &udid).await;
    }

    Err("No action specified. Use --copy or --paste.".into())
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
