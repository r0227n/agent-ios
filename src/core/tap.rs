//! tap コマンド - 要素タップ
//!
//! ```bash
//! agent-mobile tap @e1              # ref でタップ
//! agent-mobile tap "Login"          # テキストでタップ
//! agent-mobile tap 100,200          # 座標でタップ
//! agent-mobile tap home             # ハードウェアキー
//! ```

use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::{with_client, CommandResult, DeviceArgs};

use super::ref_resolver::{self, Target};

/// tap コマンド引数
#[derive(Args, Debug)]
pub struct TapArgs {
    /// Target: @eN ref, "text", x,y coordinates, or key (home, back, enter, etc.)
    pub target: String,

    /// Duration of tap in seconds
    #[arg(long)]
    pub duration: Option<f64>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the tap command
pub async fn run(args: TapArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;
    let target = Target::parse(&args.target);

    // Handle special keys
    if let Target::Key(key) = &target {
        return execute_key(platform, args.device.udid.as_deref(), key).await;
    }

    // Get coordinates from target
    let (x, y) = resolve_coords(&target, platform, args.device.udid.as_deref()).await?;

    // Execute tap
    execute_tap(platform, args.device.udid.as_deref(), x, y, args.duration).await
}

/// Resolve target to coordinates
pub async fn resolve_coords(
    target: &Target,
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
            // Take a fresh snapshot
            let snapshot = take_snapshot(platform, udid).await?;
            let element = ref_resolver::resolve_from_snapshot(&snapshot, target)?;
            Ok(element.center())
        }
        Target::Key(_) => Err("Cannot resolve key to coordinates".into()),
    }
}

/// Resolve special position to coordinates
pub async fn resolve_position(
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
pub async fn get_screen_size(platform: Platform, udid: Option<&str>) -> CommandResult<(f64, f64)> {
    match platform {
        Platform::Ios => {
            // Get screen size from snapshot's root element
            get_ios_screen_size_from_snapshot(udid).await
        }
        Platform::Android => {
            // Use adb to get actual screen size
            let (w, h) = agent_mobile_platform_android::adb::input::get_screen_size(udid).await?;
            Ok((w as f64, h as f64))
        }
    }
}

/// Get iOS screen size from snapshot's root element
async fn get_ios_screen_size_from_snapshot(udid: Option<&str>) -> CommandResult<(f64, f64)> {
    use crate::helpers::with_client;

    with_client(udid, |mut client| async move {
        let json_str = client.accessibility_info(None, false).await?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;

        // The root element's frame represents the screen bounds
        if let Some(frame) = json.get("frame") {
            if let (Some(w), Some(h)) = (
                frame.get("width").and_then(|v| v.as_f64()),
                frame.get("height").and_then(|v| v.as_f64()),
            ) {
                if w > 0.0 && h > 0.0 {
                    return Ok((w, h));
                }
            }
        }

        Err("Could not determine screen size from accessibility info".into())
    })
    .await
}

/// Take a fresh snapshot
pub async fn take_snapshot(
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<crate::snapshot::types::Snapshot> {
    use crate::snapshot::types::Snapshot;
    use chrono::Utc;

    match platform {
        Platform::Ios => {
            let udid = udid.map(|s| s.to_string());
            with_client(udid.as_deref(), |mut client| async move {
                let json_str = client.accessibility_info(None, true).await?;
                let json: serde_json::Value = serde_json::from_str(&json_str)?;
                let raw_elements = agent_mobile_platform_ios::snapshot::extract_ios_elements(&json);
                let elements = crate::snapshot::ref_generator::generate_refs(&raw_elements);
                Ok(Snapshot {
                    snapshot_id: format!("snap_{}", nanoid::nanoid!(8)),
                    timestamp: Utc::now(),
                    elements,
                })
            })
            .await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::uiautomator;

            let xml = uiautomator::dump_ui(udid).await?;
            let accessibility_elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let raw_elements = agent_mobile_platform_android::snapshot::extract_android_elements(
                &accessibility_elements,
            );
            let elements = crate::snapshot::ref_generator::generate_refs(&raw_elements);
            Ok(Snapshot {
                snapshot_id: format!("snap_{}", nanoid::nanoid!(8)),
                timestamp: Utc::now(),
                elements,
            })
        }
    }
}

/// Execute tap gesture
pub async fn execute_tap(
    platform: Platform,
    udid: Option<&str>,
    x: f64,
    y: f64,
    duration: Option<f64>,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::idb::hid::events;

            with_client(udid, |mut client| async move {
                let events = events::tap_to_events(x, y, duration);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::input;
            input::tap(udid, x, y).await?;
            Ok(())
        }
    }
}

/// Execute key press
async fn execute_key(platform: Platform, udid: Option<&str>, key: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::idb::hid::events;
            use agent_mobile_platform_ios::proto::idb::hid_event::HidButtonType;

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
            use agent_mobile_platform_android::adb::input::{self, keycodes};

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
