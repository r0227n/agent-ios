//! App command implementation.
//!
//! Provides cross-platform application management including launch,
//! terminate, install, uninstall, and list operations.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::helpers::{CommandResult, DeviceArgs, DeviceOutputArgs, OutputFormat};
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
        device_output: DeviceOutputArgs,
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
        device_output: DeviceOutputArgs,
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
        device_output: DeviceOutputArgs,
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
                &device_output.output,
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
                &device_output.output,
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
                &device_output.output,
            )
            .await
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
