//! App command implementation.
//!
//! Provides cross-platform application management including launch,
//! terminate, install, uninstall, and list operations.

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand};
use serde::Serialize;
use tempfile::TempDir;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::{DeviceArgs, DeviceFormatArgs};
use crate::helpers::format::OutputFormat;

/// App command arguments.
#[derive(Args, Debug)]
pub struct AppArgs {
    /// App management action to execute.
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

        /// Device selection and output formatting options.
        #[command(flatten)]
        device_output: DeviceFormatArgs,
    },

    /// Terminate a running app.
    Terminate {
        /// Bundle ID (iOS) or package name (Android).
        bundle_id: String,

        /// Device selection options.
        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Install an app from path (.app/.ipa for iOS, .apk for Android).
    Install {
        /// Path to the app file (.app, .ipa, or .apk).
        path: String,

        /// Device selection and output formatting options.
        #[command(flatten)]
        device_output: DeviceFormatArgs,
    },

    /// Uninstall an app.
    Uninstall {
        /// Bundle ID (iOS) or package name (Android).
        bundle_id: String,

        /// Device selection options.
        #[command(flatten)]
        device: DeviceArgs,
    },

    /// List installed apps.
    List {
        /// Device selection and output formatting options.
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

        /// Device selection options.
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

        /// Device selection options.
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

        /// Device selection options.
        #[command(flatten)]
        device: DeviceArgs,
    },
}

/// Unified app info for output.
#[derive(Debug, Serialize)]
pub struct UnifiedAppInfo {
    /// Bundle identifier or Android package name.
    pub bundle_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Human-readable application name, when available.
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Marketing version string, when available.
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Installation type reported by the platform, when available.
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

/// Execute the app command.
pub async fn run(args: AppArgs, resolved_udid: Option<String>) -> CommandResult {
    match args.command {
        AppCommands::Launch {
            bundle_id,
            mut device_output,
        } => {
            // Apply session UDID if not explicitly set
            if device_output.udid.is_none() {
                device_output.udid = resolved_udid;
            }
            let platform = match device_output.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
            execute_launch(
                platform,
                device_output.udid.as_deref(),
                &bundle_id,
                &device_output.format,
            )
            .await
        }
        AppCommands::Terminate {
            bundle_id,
            mut device,
        } => {
            if device.udid.is_none() {
                device.udid = resolved_udid;
            }
            let platform = match device.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
            execute_terminate(platform, device.udid.as_deref(), &bundle_id).await
        }
        AppCommands::Install {
            path,
            mut device_output,
        } => {
            if device_output.udid.is_none() {
                device_output.udid = resolved_udid;
            }
            let platform = match device_output.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
            execute_install(
                platform,
                device_output.udid.as_deref(),
                &path,
                &device_output.format,
            )
            .await
        }
        AppCommands::Uninstall {
            bundle_id,
            mut device,
        } => {
            if device.udid.is_none() {
                device.udid = resolved_udid;
            }
            let platform = match device.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
            execute_uninstall(platform, device.udid.as_deref(), &bundle_id).await
        }
        AppCommands::List { mut device_output } => {
            if device_output.udid.is_none() {
                device_output.udid = resolved_udid;
            }
            let platform = match device_output.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
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
            mut device,
        } => {
            if device.udid.is_none() {
                device.udid = resolved_udid;
            }
            let platform = match device.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
            let udid = match &device.udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_grant(platform, &udid, &bundle, &permission).await
        }
        AppCommands::Revoke {
            permission,
            bundle,
            mut device,
        } => {
            if device.udid.is_none() {
                device.udid = resolved_udid;
            }
            let platform = match device.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
            let udid = match &device.udid {
                Some(u) => u.clone(),
                None => get_default_udid(platform).await?,
            };
            execute_revoke(platform, &udid, &bundle, &permission).await
        }
        AppCommands::Reset {
            permission,
            bundle,
            mut device,
        } => {
            if device.udid.is_none() {
                device.udid = resolved_udid;
            }
            let platform = match device.udid.as_deref() {
                Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
                None => DeviceResolver::detect_platform().await?,
            };
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
            let udid = resolve_ios_udid(udid)?;
            let pid = agent_mobile_platform_ios::coresim::launch_app(&udid, bundle_id)?;
            crate::session::app_context::set_active_app(&udid, bundle_id)?;
            println!("Launched {} (pid: {})", bundle_id, pid);
            Ok(())
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::app;
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
            let udid = resolve_ios_udid(udid)?;
            agent_mobile_platform_ios::coresim::terminate_app(&udid, bundle_id)?;
            crate::session::app_context::clear_active_app(&udid)?;
            println!("Terminated {}", bundle_id);
            Ok(())
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::app;
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
    _output: &OutputFormat,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            let udid = resolve_ios_udid(udid)?;
            let (install_path, _temp_dir) = resolve_ios_install_source(path)?;
            println!("Installing {}...", install_path.display());
            agent_mobile_platform_ios::simctl::install_app(
                &udid,
                install_path_to_str(&install_path)?,
            )?;
            println!("Installation complete");
            Ok(())
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::app;
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
            let udid = resolve_ios_udid(udid)?;
            agent_mobile_platform_ios::simctl::uninstall_app(&udid, bundle_id)?;
            println!("Uninstalled {}", bundle_id);
            Ok(())
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::app;
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
            // Use simctl directly (no XCUITest Runner needed)
            let udid = match udid {
                Some(u) => u.to_string(),
                None => agent_mobile_platform_ios::simctl::get_booted_simulator()?.udid,
            };
            let apps = agent_mobile_platform_ios::simctl::list_apps(&udid)?;
            let unified: Vec<UnifiedAppInfo> = apps
                .iter()
                .map(|app| UnifiedAppInfo {
                    bundle_id: app.bundle_id.clone(),
                    name: if app.name.is_empty() {
                        None
                    } else {
                        Some(app.name.clone())
                    },
                    version: if app.version.is_empty() {
                        None
                    } else {
                        Some(app.version.clone())
                    },
                    install_type: if app.app_type.is_empty() {
                        None
                    } else {
                        Some(app.app_type.clone())
                    },
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
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::app;

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
            let booted = agent_mobile_platform_ios::simctl::get_booted_simulator()?;
            Ok(booted.udid)
        }
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

            // Use simctl for permission management
            use agent_mobile_platform_ios::simctl::management;
            management::privacy_grant(udid, permission, bundle_id)?;
            println!("Granted {} to {}", permission, bundle_id);
            Ok(())
        }
        Platform::Android => {
            let android_perm = android_permission_name(permission)?;

            agent_mobile_platform_android::grant_permission(Some(udid), bundle_id, android_perm)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

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
            // Use simctl for permission management
            use agent_mobile_platform_ios::simctl::management;
            management::privacy_revoke(udid, permission, bundle_id)?;
            println!("Revoked {} from {}", permission, bundle_id);
            Ok(())
        }
        Platform::Android => {
            let android_perm = android_permission_name(permission)?;

            agent_mobile_platform_android::revoke_permission(Some(udid), bundle_id, android_perm)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

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
            use agent_mobile_platform_ios::simctl::management;
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

fn resolve_ios_udid(
    udid: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match udid {
        Some(udid) => Ok(udid.to_string()),
        None => Ok(agent_mobile_platform_ios::simctl::get_booted_simulator()?.udid),
    }
}

fn resolve_ios_install_source(
    path: &str,
) -> Result<(PathBuf, Option<TempDir>), Box<dyn std::error::Error + Send + Sync>> {
    let source = PathBuf::from(path);

    if !source.exists() {
        return Err(format!("iOS app not found: {}", path).into());
    }

    if source.extension().and_then(|ext| ext.to_str()) == Some("app") {
        return Ok((source, None));
    }

    if source.extension().and_then(|ext| ext.to_str()) == Some("ipa") {
        let temp_dir = tempfile::tempdir()?;
        let status = Command::new("ditto")
            .args([
                "-x",
                "-k",
                install_path_to_str(&source)?,
                install_path_to_str(temp_dir.path())?,
            ])
            .status()?;

        if !status.success() {
            return Err(format!("Failed to extract iOS archive: {}", path).into());
        }

        let app_bundle = find_ios_app_bundle(temp_dir.path())?;
        return Ok((app_bundle, Some(temp_dir)));
    }

    Err(format!(
        "Unsupported iOS app format: {}. Expected .app or .ipa",
        path
    )
    .into())
}

fn find_ios_app_bundle(
    extracted_root: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let payload = extracted_root.join("Payload");
    if !payload.exists() {
        return Err("Invalid iOS archive: missing Payload directory".into());
    }

    for entry in std::fs::read_dir(&payload)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("app") {
            return Ok(path);
        }
    }

    Err("Invalid iOS archive: no .app bundle found in Payload".into())
}

fn install_path_to_str(path: &Path) -> Result<&str, Box<dyn std::error::Error + Send + Sync>> {
    path.to_str()
        .ok_or_else(|| format!("Invalid path encoding: {}", path.display()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_ios_install_source_missing_path() {
        let result = resolve_ios_install_source("/nonexistent/app.ipa");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_ios_install_source_unsupported_extension() {
        let temp_dir = tempfile::tempdir().unwrap();
        let unsupported = temp_dir.path().join("app.zip");
        std::fs::write(&unsupported, b"dummy").unwrap();

        let result = resolve_ios_install_source(unsupported.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_find_ios_app_bundle_finds_payload_app() {
        let temp_dir = tempfile::tempdir().unwrap();
        let payload = temp_dir.path().join("Payload");
        std::fs::create_dir(&payload).unwrap();
        let app = payload.join("MockApp.app");
        std::fs::create_dir(&app).unwrap();

        let found = find_ios_app_bundle(temp_dir.path()).unwrap();
        assert_eq!(found, app);
    }
}
