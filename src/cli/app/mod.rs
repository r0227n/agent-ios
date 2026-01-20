//! App command implementation.
//!
//! Provides cross-platform application management including launch,
//! terminate, install, uninstall, and list operations.

use clap::Args;
use serde::Serialize;

use crate::cli::helpers::{CommandResult, OutputFormat};
use crate::types::Platform;

/// App command arguments.
#[derive(Args, Debug)]
pub struct AppArgs {
    /// Launch an app by bundle ID (iOS) or package name (Android).
    #[arg(long)]
    pub launch: Option<String>,

    /// Terminate a running app.
    #[arg(long)]
    pub terminate: Option<String>,

    /// Install an app from path (.app/.ipa for iOS, .apk for Android).
    #[arg(long)]
    pub install: Option<String>,

    /// Uninstall an app.
    #[arg(long)]
    pub uninstall: Option<String>,

    /// List installed apps.
    #[arg(long)]
    pub list: bool,

    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,

    /// Output format (human or json).
    #[arg(short = 'o', long, value_enum, default_value = "human")]
    pub output: OutputFormat,
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

/// Execute the app command.
pub async fn run(args: AppArgs) -> CommandResult {
    let platform = match &args.platform {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?,
        None => detect_platform().await?,
    };

    if let Some(ref bundle_id) = args.launch {
        return execute_launch(platform, &args, bundle_id).await;
    }

    if let Some(ref bundle_id) = args.terminate {
        return execute_terminate(platform, &args, bundle_id).await;
    }

    if let Some(ref path) = args.install {
        return execute_install(platform, &args, path).await;
    }

    if let Some(ref bundle_id) = args.uninstall {
        return execute_uninstall(platform, &args, bundle_id).await;
    }

    if args.list {
        return execute_list(platform, &args).await;
    }

    Err("No action specified. Use --launch, --terminate, --install, --uninstall, or --list.".into())
}

/// Execute app launch.
async fn execute_launch(platform: Platform, args: &AppArgs, bundle_id: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::grpc::LaunchConfig;
            use std::collections::HashMap;
            use tokio::sync::watch;

            let (_tx, stop_rx) = watch::channel(false);

            with_client(args.udid.as_deref(), |mut client| async move {
                let config = LaunchConfig {
                    bundle_id: bundle_id.to_string(),
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
            app::launch(args.udid.as_deref(), bundle_id).await?;
            println!("Launched {}", bundle_id);
            Ok(())
        }
    }
}

/// Execute app terminate.
async fn execute_terminate(platform: Platform, args: &AppArgs, bundle_id: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                client.terminate(bundle_id).await?;
                println!("Terminated {}", bundle_id);
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::app;
            app::terminate(args.udid.as_deref(), bundle_id).await?;
            println!("Terminated {}", bundle_id);
            Ok(())
        }
    }
}

/// Execute app install.
async fn execute_install(platform: Platform, args: &AppArgs, path: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                let mut stream = client.install(path, false, false, None).await?;
                while let Some(response) = stream.message().await? {
                    let name = &response.name;
                    if !name.is_empty() {
                        let progress = response.progress;
                        if args.output.is_json() {
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
            app::install(args.udid.as_deref(), path, true).await?;
            println!("Installation complete");
            Ok(())
        }
    }
}

/// Execute app uninstall.
async fn execute_uninstall(platform: Platform, args: &AppArgs, bundle_id: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                client.uninstall(bundle_id).await?;
                println!("Uninstalled {}", bundle_id);
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::app;
            app::uninstall(args.udid.as_deref(), bundle_id).await?;
            println!("Uninstalled {}", bundle_id);
            Ok(())
        }
    }
}

/// Execute list apps.
async fn execute_list(platform: Platform, args: &AppArgs) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
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

                if args.output.is_json() {
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

            let apps = app::list_packages(args.udid.as_deref(), false).await?;
            let unified: Vec<UnifiedAppInfo> = apps
                .iter()
                .map(|app| UnifiedAppInfo {
                    bundle_id: app.package_name.clone(),
                    name: None,
                    version: app.version_name.clone(),
                    install_type: None,
                })
                .collect();

            if args.output.is_json() {
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
