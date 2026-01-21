//! App command implementation.
//!
//! Provides cross-platform application management including launch,
//! terminate, install, uninstall, and list operations.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::helpers::{CommandResult, DeviceArgs, DeviceFormatArgs, OutputFormat};
use crate::types::Platform;

/// App command arguments.
#[derive(Args, Debug)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: AppCommands,
}

/// App subcommands.
#[derive(Subcommand, Debug)]
pub enum AppCommands {
    /// Launch an app by bundle ID (iOS) or package name (Android).
    Launch {
        /// Bundle ID (iOS) or package name (Android).
        bundle_id: String,

        #[command(flatten)]
        device_output: DeviceFormatArgs,
    },

    /// Terminate a running app.
    Terminate {
        /// Bundle ID (iOS) or package name (Android).
        bundle_id: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Install an app from path (.app/.ipa for iOS, .apk for Android).
    Install {
        /// Path to the app file (.app, .ipa, or .apk).
        path: String,

        #[command(flatten)]
        device_output: DeviceFormatArgs,
    },

    /// Uninstall an app.
    Uninstall {
        /// Bundle ID (iOS) or package name (Android).
        bundle_id: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// List installed apps.
    List {
        #[command(flatten)]
        device_output: DeviceFormatArgs,
    },

    /// Grant a permission to an app.
    Grant {
        /// Permission name (e.g., camera, location, contacts).
        permission: String,

        /// Bundle ID / package name of the app.
        #[arg(short = 'b', long)]
        bundle: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Revoke a permission from an app.
    Revoke {
        /// Permission name (e.g., camera, location, contacts).
        permission: String,

        /// Bundle ID / package name of the app.
        #[arg(short = 'b', long)]
        bundle: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Reset a permission for an app.
    Reset {
        /// Permission name (e.g., camera, location, contacts).
        permission: String,

        /// Bundle ID / package name of the app.
        #[arg(short = 'b', long)]
        bundle: String,

        #[command(flatten)]
        device: DeviceArgs,
    },
}

/// Unified app info for output.
#[derive(Debug, Serialize)]
pub struct UnifiedAppInfo {
    pub bundle_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_type: Option<String>,
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

/// Execute the app command.
pub async fn run(args: AppArgs) -> CommandResult {
    match args.command {
        AppCommands::Launch {
            bundle_id,
            device_output,
        } => {
            let platform = resolve_platform(device_output.platform.as_deref()).await?;
            execute_launch(
                platform,
                device_output.udid.as_deref(),
                &bundle_id,
                &device_output.format,
            )
            .await
        }
        AppCommands::Terminate { bundle_id, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_terminate(platform, device.udid.as_deref(), &bundle_id).await
        }
        AppCommands::Install {
            path,
            device_output,
        } => {
            let platform = resolve_platform(device_output.platform.as_deref()).await?;
            execute_install(
                platform,
                device_output.udid.as_deref(),
                &path,
                &device_output.format,
            )
            .await
        }
        AppCommands::Uninstall { bundle_id, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_uninstall(platform, device.udid.as_deref(), &bundle_id).await
        }
        AppCommands::List { device_output } => {
            let platform = resolve_platform(device_output.platform.as_deref()).await?;
            execute_list(
                platform,
                device_output.udid.as_deref(),
                &device_output.format,
            )
            .await
        }
        AppCommands::Grant {
            permission,
            bundle,
            device,
        } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            let udid = match &device.udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_grant(platform, &udid, &bundle, &permission).await
        }
        AppCommands::Revoke {
            permission,
            bundle,
            device,
        } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            let udid = match &device.udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_revoke(platform, &udid, &bundle, &permission).await
        }
        AppCommands::Reset {
            permission,
            bundle,
            device,
        } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            let udid = match &device.udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_reset(platform, &udid, &bundle, &permission).await
        }
    }
}

/// Execute app launch.
async fn execute_launch(
    platform: Platform,
    udid: Option<&str>,
    bundle_id: &str,
    _output: &OutputFormat,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::grpc::LaunchConfig;
            use std::collections::HashMap;
            use tokio::sync::watch;

            let (_tx, stop_rx) = watch::channel(false);
            let bundle_id = bundle_id.to_string();

            with_client(udid, |mut client| async move {
                let config = LaunchConfig {
                    bundle_id: bundle_id.clone(),
                    app_args: Vec::new(),
                    env: HashMap::new(),
                    foreground_if_running: true,
                    wait_for_debugger: false,
                };
                let pid = client.launch(config, false, stop_rx).await?;
                if let Some(p) = pid {
                    println!("Launched {} (PID: {})", bundle_id, p);
                } else {
                    println!("Launched {}", bundle_id);
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::app;
            app::launch(udid, bundle_id).await?;
            println!("Launched {}", bundle_id);
            Ok(())
        }
    }
}

/// Execute app terminate.
async fn execute_terminate(
    platform: Platform,
    udid: Option<&str>,
    bundle_id: &str,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            let bundle_id = bundle_id.to_string();

            with_client(udid, |mut client| async move {
                client.terminate(&bundle_id).await?;
                println!("Terminated {}", bundle_id);
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::app;
            app::terminate(udid, bundle_id).await?;
            println!("Terminated {}", bundle_id);
            Ok(())
        }
    }
}

/// Execute app install.
async fn execute_install(
    platform: Platform,
    udid: Option<&str>,
    path: &str,
    output: &OutputFormat,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            let path = path.to_string();
            let output = output.clone();

            with_client(udid, |mut client| async move {
                let mut stream = client.install(&path, false, false, None).await?;
                while let Some(response) = stream.message().await? {
                    let name = &response.name;
                    if !name.is_empty() {
                        let progress = response.progress;
                        if output.is_json() {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "name": name,
                                    "progress": progress
                                })
                            );
                        } else if progress > 0.0 {
                            println!("Installing {}: {:.0}%", name, progress * 100.0);
                        }
                    }
                }
                println!("Installation complete");
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::app;
            println!("Installing {}...", path);
            app::install(udid, path, true).await?;
            println!("Installation complete");
            Ok(())
        }
    }
}

/// Execute app uninstall.
async fn execute_uninstall(
    platform: Platform,
    udid: Option<&str>,
    bundle_id: &str,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            let bundle_id = bundle_id.to_string();

            with_client(udid, |mut client| async move {
                client.uninstall(&bundle_id).await?;
                println!("Uninstalled {}", bundle_id);
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::app;
            app::uninstall(udid, bundle_id).await?;
            println!("Uninstalled {}", bundle_id);
            Ok(())
        }
    }
}

/// Execute list apps.
async fn execute_list(
    platform: Platform,
    udid: Option<&str>,
    output: &OutputFormat,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            let output = output.clone();

            with_client(udid, |mut client| async move {
                let apps = client.list_apps().await?;
                let unified: Vec<UnifiedAppInfo> = apps
                    .iter()
                    .map(|app| UnifiedAppInfo {
                        bundle_id: app.bundle_id.clone(),
                        name: if app.name.is_empty() {
                            None
                        } else {
                            Some(app.name.clone())
                        },
                        version: None,      // iOS doesn't provide this in list_apps
                        install_type: None, // Simplify - skip install_type for now
                    })
                    .collect();

                if output.is_json() {
                    println!("{}", serde_json::to_string_pretty(&unified)?);
                } else {
                    println!("Installed Apps ({}):", unified.len());
                    println!("{:-<60}", "");
                    for app in &unified {
                        if let Some(name) = &app.name {
                            println!("{} ({})", name, app.bundle_id);
                        } else {
                            println!("{}", app.bundle_id);
                        }
                    }
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::app;

            let apps = app::list_packages(udid, false).await?;
            let unified: Vec<UnifiedAppInfo> = apps
                .iter()
                .map(|app| UnifiedAppInfo {
                    bundle_id: app.package_name.clone(),
                    name: None,
                    version: app.version_name.clone(),
                    install_type: None,
                })
                .collect();

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&unified)?);
            } else {
                println!("Installed Apps ({}):", unified.len());
                println!("{:-<60}", "");
                for app in &unified {
                    println!("{}", app.bundle_id);
                }
            }
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
