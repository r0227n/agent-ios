//! Privacy (permissions) command implementation.
//!
//! Provides cross-platform permission management including grant, revoke,
//! and reset operations.

use clap::Args;

use crate::cli::helpers::CommandResult;
use crate::types::Platform;

/// Privacy command arguments.
#[derive(Args, Debug)]
pub struct PrivacyArgs {
    /// Grant a permission.
    #[arg(long)]
    pub grant: Option<String>,

    /// Revoke a permission.
    #[arg(long)]
    pub revoke: Option<String>,

    /// Reset a permission.
    #[arg(long)]
    pub reset: Option<String>,

    /// Bundle ID / package name of the app.
    #[arg(long, short = 'b')]
    pub bundle: String,

    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,
}

/// Valid iOS permission services.
const IOS_PERMISSIONS: &[&str] = &[
    "all",
    "calendar",
    "contacts-limited",
    "contacts",
    "location",
    "location-always",
    "photos-add",
    "photos",
    "media-library",
    "microphone",
    "motion",
    "reminders",
    "siri",
    "speech-recognition",
    "camera",
    "faceid",
    "homekit",
    "health",
];

/// Valid Android permission names.
const ANDROID_PERMISSIONS: &[(&str, &str)] = &[
    ("camera", "android.permission.CAMERA"),
    ("location", "android.permission.ACCESS_FINE_LOCATION"),
    (
        "location-coarse",
        "android.permission.ACCESS_COARSE_LOCATION",
    ),
    ("contacts", "android.permission.READ_CONTACTS"),
    ("calendar", "android.permission.READ_CALENDAR"),
    ("storage", "android.permission.READ_EXTERNAL_STORAGE"),
    ("microphone", "android.permission.RECORD_AUDIO"),
    ("phone", "android.permission.CALL_PHONE"),
    ("sms", "android.permission.READ_SMS"),
];

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

/// Execute the privacy command.
pub async fn run(args: PrivacyArgs) -> CommandResult {
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

    if let Some(ref permission) = args.grant {
        return execute_grant(platform, &udid, &args.bundle, permission).await;
    }

    if let Some(ref permission) = args.revoke {
        return execute_revoke(platform, &udid, &args.bundle, permission).await;
    }

    if let Some(ref permission) = args.reset {
        return execute_reset(platform, &udid, &args.bundle, permission).await;
    }

    Err("No action specified. Use --grant, --revoke, or --reset.".into())
}

/// Execute permission grant.
async fn execute_grant(
    platform: Platform,
    udid: &str,
    bundle_id: &str,
    permission: &str,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            // Validate permission
            if !IOS_PERMISSIONS.contains(&permission) {
                return Err(format!(
                    "Invalid iOS permission: {}. Valid permissions: {:?}",
                    permission, IOS_PERMISSIONS
                )
                .into());
            }

            // Try idb gRPC first, fall back to simctl
            use crate::cli::helpers::with_client;

            let result = with_client(Some(udid), |mut client| async move {
                let perm_id = ios_permission_to_id(permission);
                client.approve(bundle_id, vec![perm_id], None).await
            })
            .await;

            match result {
                Ok(_) => {
                    println!("Granted {} to {}", permission, bundle_id);
                    Ok(())
                }
                Err(_) => {
                    // Fall back to simctl
                    use crate::platform::ios::simctl::management;
                    management::privacy_grant(udid, permission, bundle_id)?;
                    println!("Granted {} to {} (via simctl)", permission, bundle_id);
                    Ok(())
                }
            }
        }
        Platform::Android => {
            use tokio::process::Command;

            let android_perm = android_permission_name(permission)?;

            let mut cmd = Command::new("adb");
            cmd.args(["-s", udid, "shell", "pm", "grant", bundle_id, android_perm]);

            let output = cmd.output().await?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to grant permission: {}", stderr).into());
            }

            println!("Granted {} to {}", permission, bundle_id);
            Ok(())
        }
    }
}

/// Execute permission revoke.
async fn execute_revoke(
    platform: Platform,
    udid: &str,
    bundle_id: &str,
    permission: &str,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            // Try idb gRPC first, fall back to simctl
            use crate::cli::helpers::with_client;

            let result = with_client(Some(udid), |mut client| async move {
                let perm_id = ios_permission_to_id(permission);
                client.revoke(bundle_id, vec![perm_id], None).await
            })
            .await;

            match result {
                Ok(_) => {
                    println!("Revoked {} from {}", permission, bundle_id);
                    Ok(())
                }
                Err(_) => {
                    // Fall back to simctl
                    use crate::platform::ios::simctl::management;
                    management::privacy_revoke(udid, permission, bundle_id)?;
                    println!("Revoked {} from {} (via simctl)", permission, bundle_id);
                    Ok(())
                }
            }
        }
        Platform::Android => {
            use tokio::process::Command;

            let android_perm = android_permission_name(permission)?;

            let mut cmd = Command::new("adb");
            cmd.args(["-s", udid, "shell", "pm", "revoke", bundle_id, android_perm]);

            let output = cmd.output().await?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to revoke permission: {}", stderr).into());
            }

            println!("Revoked {} from {}", permission, bundle_id);
            Ok(())
        }
    }
}

/// Execute permission reset.
async fn execute_reset(
    platform: Platform,
    udid: &str,
    bundle_id: &str,
    permission: &str,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::platform::ios::simctl::management;
            management::privacy_reset(udid, permission, bundle_id)?;
            println!("Reset {} for {}", permission, bundle_id);
            Ok(())
        }
        Platform::Android => {
            // Android doesn't have a reset concept, just revoke
            execute_revoke(platform, udid, bundle_id, permission).await
        }
    }
}

/// Convert iOS permission name to idb permission ID.
fn ios_permission_to_id(permission: &str) -> i32 {
    // These IDs match the idb.proto Permission enum
    match permission {
        "photos" => 1,
        "camera" => 2,
        "contacts" => 3,
        "url" => 4,
        "location" => 5,
        "notification" => 6,
        "microphone" => 7,
        _ => 0, // Unknown
    }
}

/// Convert permission short name to Android permission string.
fn android_permission_name(
    short_name: &str,
) -> Result<&'static str, Box<dyn std::error::Error + Send + Sync>> {
    for (short, full) in ANDROID_PERMISSIONS {
        if *short == short_name {
            return Ok(full);
        }
    }

    // Check if it's already a full permission name
    if short_name.starts_with("android.permission.") {
        // Return a static string approximation
        return Err(format!(
            "Use short permission names: {:?}",
            ANDROID_PERMISSIONS
                .iter()
                .map(|(s, _)| s)
                .collect::<Vec<_>>()
        )
        .into());
    }

    Err(format!(
        "Unknown permission: {}. Valid permissions: {:?}",
        short_name,
        ANDROID_PERMISSIONS
            .iter()
            .map(|(s, _)| s)
            .collect::<Vec<_>>()
    )
    .into())
}
