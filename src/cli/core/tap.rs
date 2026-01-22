//! tap コマンド - 要素タップ
//!
//! ```bash
//! agent-mobile tap @e1              # ref でタップ
//! agent-mobile tap "Login"          # テキストでタップ
//! agent-mobile tap 100,200          # 座標でタップ
//! agent-mobile tap home             # ハードウェアキー
//! agent-mobile tap @e1 --snapshot snap.json
//! ```

use std::path::PathBuf;

use clap::Args;

use crate::cli::helpers::{with_client, CommandResult, DeviceArgs};
use crate::types::Platform;

use super::ref_resolver::{self, Target};

/// tap コマンド引数
#[derive(Args, Debug)]
pub struct TapArgs {
    /// Target: @eN ref, "text", x,y coordinates, or key (home, back, enter, etc.)
    pub target: String,

    /// Snapshot file to use (instead of taking fresh snapshot)
    #[arg(long)]
    pub snapshot: Option<PathBuf>,

    /// Duration of tap in seconds
    #[arg(long)]
    pub duration: Option<f64>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the tap command
pub async fn run(args: TapArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;
    let target = Target::parse(&args.target);

    // Handle special keys
    if let Target::Key(key) = &target {
        return execute_key(platform, args.device.udid.as_deref(), key).await;
    }

    // Get coordinates from target
    let (x, y) = resolve_coords(
        &target,
        args.snapshot.as_ref(),
        platform,
        args.device.udid.as_deref(),
    )
    .await?;

    // Execute tap
    execute_tap(platform, args.device.udid.as_deref(), x, y, args.duration).await
}

/// Resolve target to coordinates
async fn resolve_coords(
    target: &Target,
    snapshot_path: Option<&PathBuf>,
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<(f64, f64)> {
    match target {
        Target::Coords(x, y) => Ok((*x, *y)),
        Target::Position(pos) => {
            // Handle special positions like "center"
            resolve_position(pos, platform, udid).await
        }
        Target::Ref(_) | Target::Text(_) => {
            // Need to resolve from snapshot
            let snapshot = if let Some(path) = snapshot_path {
                ref_resolver::load_snapshot_from_file(path)?
            } else {
                // Take a fresh snapshot
                take_snapshot(platform, udid).await?
            };

            let element = ref_resolver::resolve_from_snapshot(&snapshot, target)?;
            Ok(element.center())
        }
        Target::Key(_) => Err("Cannot resolve key to coordinates".into()),
    }
}

/// Resolve special position to coordinates
async fn resolve_position(
    position: &str,
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<(f64, f64)> {
    // Get screen dimensions
    let (width, height) = get_screen_size(platform, udid).await?;

    match position {
        "center" => Ok((width / 2.0, height / 2.0)),
        _ => Err(format!("Unknown position: {}", position).into()),
    }
}

/// Get screen dimensions
async fn get_screen_size(platform: Platform, _udid: Option<&str>) -> CommandResult<(f64, f64)> {
    match platform {
        Platform::Ios => {
            // Default iPhone screen size in points (iPhone 13/14/15)
            Ok((390.0, 844.0))
        }
        Platform::Android => {
            // Default Android screen size in pixels
            Ok((1080.0, 2340.0))
        }
    }
}

/// Take a fresh snapshot
pub async fn take_snapshot(
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<crate::cli::snapshot::types::Snapshot> {
    use crate::cli::snapshot::types::Snapshot;
    use chrono::Utc;

    match platform {
        Platform::Ios => {
            let udid = udid.map(|s| s.to_string());
            with_client(udid.as_deref(), |mut client| async move {
                let json_str = client.accessibility_info(None, true).await?;
                let json: serde_json::Value = serde_json::from_str(&json_str)?;
                let raw_elements = crate::cli::snapshot::extractor::extract_ios_elements(&json);
                let elements = crate::cli::snapshot::ref_generator::generate_refs(&raw_elements);
                Ok(Snapshot {
                    snapshot_id: format!("snap_{}", nanoid::nanoid!(8)),
                    timestamp: Utc::now(),
                    elements,
                })
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let xml = uiautomator::dump_ui(udid).await?;
            let accessibility_elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let raw_elements =
                crate::cli::snapshot::extractor::extract_android_elements(&accessibility_elements);
            let elements = crate::cli::snapshot::ref_generator::generate_refs(&raw_elements);
            Ok(Snapshot {
                snapshot_id: format!("snap_{}", nanoid::nanoid!(8)),
                timestamp: Utc::now(),
                elements,
            })
        }
    }
}

/// Execute tap gesture
async fn execute_tap(
    platform: Platform,
    udid: Option<&str>,
    x: f64,
    y: f64,
    duration: Option<f64>,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::idb::hid::events;

            with_client(udid, |mut client| async move {
                let events = events::tap_to_events(x, y, duration);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            input::tap(udid, x, y).await?;
            Ok(())
        }
    }
}

/// Execute key press
async fn execute_key(platform: Platform, udid: Option<&str>, key: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::idb::hid::events;
            use crate::grpc::idb::hid_event::HidButtonType;

            // Check if it's a button (home, lock, etc.) or a keyboard key
            let key_lower = key.to_lowercase();

            // Hardware buttons
            if let Some(button_type) = match key_lower.as_str() {
                "home" => Some(HidButtonType::Home),
                "lock" | "power" => Some(HidButtonType::Lock),
                "siri" => Some(HidButtonType::Siri),
                _ => None,
            } {
                return with_client(udid, |mut client| async move {
                    let events = events::button_to_events(button_type, None);
                    client.hid(events).await?;
                    Ok(())
                })
                .await;
            }

            // Keyboard keys
            let keycode = match key_lower.as_str() {
                "enter" | "return" => 40,
                "escape" | "esc" => 41,
                "delete" | "backspace" => 42,
                "tab" => 43,
                "space" => 44,
                "up" => 82,
                "down" => 81,
                "left" => 80,
                "right" => 79,
                _ => {
                    return Err(format!(
                        "Unknown key: {}. Valid keys: home, lock, siri, enter, tab, space, escape, delete, up, down, left, right",
                        key
                    )
                    .into())
                }
            };

            with_client(udid, |mut client| async move {
                let events = events::key_to_events(keycode, None);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input::{self, keycodes};

            let key_lower = key.to_lowercase();
            let keycode = match key_lower.as_str() {
                "home" => keycodes::HOME,
                "back" => keycodes::BACK,
                "menu" => keycodes::MENU,
                "power" | "lock" => keycodes::POWER,
                "enter" | "return" => keycodes::ENTER,
                "delete" | "backspace" => keycodes::DEL,
                "tab" => keycodes::TAB,
                "space" => keycodes::SPACE,
                "escape" | "esc" => keycodes::ESCAPE,
                "up" => keycodes::DPAD_UP,
                "down" => keycodes::DPAD_DOWN,
                "left" => keycodes::DPAD_LEFT,
                "right" => keycodes::DPAD_RIGHT,
                "volume-up" | "volume_up" => keycodes::VOLUME_UP,
                "volume-down" | "volume_down" => keycodes::VOLUME_DOWN,
                _ => {
                    return Err(format!(
                        "Unknown Android key: {}. Valid keys: home, back, menu, power, enter, tab, space, escape, delete, up, down, left, right, volume-up, volume-down",
                        key
                    )
                    .into())
                }
            };

            input::keyevent(udid, keycode).await?;
            Ok(())
        }
    }
}

/// Detect platform based on available devices
async fn detect_platform() -> CommandResult<Platform> {
    // Try iOS first (check for companion state)
    let ios_state_path = std::path::Path::new("/tmp/idb/state");
    if ios_state_path.exists() {
        return Ok(Platform::Ios);
    }

    // Try Android (check for connected devices)
    if crate::platform::android::adb::is_adb_available() {
        let devices = crate::platform::android::adb::list_devices();
        if let Ok(devs) = devices {
            if !devs.is_empty() {
                return Ok(Platform::Android);
            }
        }
    }

    // Default to iOS
    Ok(Platform::Ios)
}

/// Resolve platform from optional string
pub async fn resolve_platform(platform_str: Option<&str>) -> CommandResult<Platform> {
    match platform_str {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() }),
        None => detect_platform().await,
    }
}
