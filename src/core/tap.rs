//! tap command - Tap an element (instant)
//!
//! ```bash
//! agent-mobile tap @e1              # Tap by ref
//! agent-mobile tap "Login"          # Tap by text
//! agent-mobile tap 100,200          # Tap by coordinates
//! agent-mobile tap home             # Hardware key
//!
//! # For long press, use the long-press command
//! agent-mobile long-press @e1 --duration 2.0
//! ```

use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;
use agent_mobile_platform_ios::xcuitest::XCUITestClient;

use crate::helpers::client::{with_xcuitest, CommandResult};
use crate::helpers::common_args::DeviceArgs;
use crate::helpers::ios::{get_ios_screen_size, get_ios_screen_size_from_client};

use super::ref_resolver::{self, ElementTarget};

/// Arguments for the tap command
#[derive(Args, Debug)]
pub struct TapArgs {
    /// Target: @eN ref, "text", x,y coordinates, or key (home, back, enter, etc.)
    pub target: String,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the tap command
pub async fn run(args: TapArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };
    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => run_android(args).await,
    }
}

async fn run_ios(args: TapArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);

    with_xcuitest(args.device.udid.as_deref(), |client| async move {
        if let ElementTarget::Key(key) = &target {
            return execute_ios_key(&client, key).await;
        }

        let (x, y) = resolve_ios_coords(&target, &client).await?;
        execute_ios_tap(&client, x, y).await
    })
    .await
}

async fn run_android(args: TapArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);

    if let ElementTarget::Key(key) = &target {
        return execute_key(Platform::Android, args.device.udid.as_deref(), key).await;
    }

    let (x, y) = resolve_coords(&target, Platform::Android, args.device.udid.as_deref()).await?;
    execute_tap(Platform::Android, args.device.udid.as_deref(), x, y).await
}

/// Resolve target to coordinates
pub async fn resolve_coords(
    target: &ElementTarget,
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<(f64, f64)> {
    match target {
        ElementTarget::Coords(x, y) => Ok((*x, *y)),
        ElementTarget::Position(pos) => {
            // Handle special positions like "center"
            resolve_position(pos, platform, udid).await
        }
        ElementTarget::Ref(_) | ElementTarget::Text(_) => {
            // Take a fresh snapshot
            let snapshot = take_snapshot(platform, udid).await?;
            let element = ref_resolver::resolve_from_snapshot(&snapshot, target)?;
            Ok(element.center())
        }
        ElementTarget::Key(_) => Err("Cannot resolve key to coordinates".into()),
    }
}

async fn resolve_ios_coords(
    target: &ElementTarget,
    client: &XCUITestClient,
) -> CommandResult<(f64, f64)> {
    match target {
        ElementTarget::Coords(x, y) => Ok((*x, *y)),
        ElementTarget::Position(pos) => resolve_ios_position(pos, client).await,
        ElementTarget::Ref(_) | ElementTarget::Text(_) => {
            let snapshot = take_ios_snapshot_with_client(client).await?;
            let element = ref_resolver::resolve_from_snapshot(&snapshot, target)?;
            Ok(element.center())
        }
        ElementTarget::Key(_) => Err("Cannot resolve key to coordinates".into()),
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

async fn resolve_ios_position(
    position: &str,
    client: &XCUITestClient,
) -> CommandResult<(f64, f64)> {
    let (width, height) = get_ios_screen_size_from_client(client).await?;

    match position {
        "center" => Ok((width / 2.0, height / 2.0)),
        _ => Err(format!("Unknown position: {}", position).into()),
    }
}

/// Get screen dimensions
pub async fn get_screen_size(platform: Platform, udid: Option<&str>) -> CommandResult<(f64, f64)> {
    match platform {
        Platform::Ios => get_ios_screen_size(udid).await,
        Platform::Android => {
            // Use adb to get actual screen size
            let (w, h) = agent_mobile_platform_android::adb::input::get_screen_size(udid).await?;
            Ok((w as f64, h as f64))
        }
    }
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
            with_xcuitest(udid, |client| async move {
                let json_str = client.accessibility_info(true).await?;
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

pub(crate) async fn take_ios_snapshot_with_client(
    client: &XCUITestClient,
) -> CommandResult<crate::snapshot::types::Snapshot> {
    use crate::snapshot::types::Snapshot;
    use chrono::Utc;

    let json_str = client.accessibility_info(true).await?;
    let json: serde_json::Value = serde_json::from_str(&json_str)?;
    let raw_elements = agent_mobile_platform_ios::snapshot::extract_ios_elements(&json);
    let elements = crate::snapshot::ref_generator::generate_refs(&raw_elements);
    Ok(Snapshot {
        snapshot_id: format!("snap_{}", nanoid::nanoid!(8)),
        timestamp: Utc::now(),
        elements,
    })
}

/// Execute tap gesture
pub async fn execute_tap(platform: Platform, udid: Option<&str>, x: f64, y: f64) -> CommandResult {
    match platform {
        Platform::Ios => {
            with_xcuitest(udid, |client| async move {
                client.tap(x, y).await?;
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

pub(crate) async fn execute_ios_tap(client: &XCUITestClient, x: f64, y: f64) -> CommandResult {
    client.tap(x, y).await?;
    Ok(())
}

/// Execute key press
async fn execute_key(platform: Platform, udid: Option<&str>, key: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            with_xcuitest(
                udid,
                |client| async move { execute_ios_key(&client, key).await },
            )
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

pub(crate) async fn execute_ios_key(client: &XCUITestClient, key: &str) -> CommandResult {
    let key_lower = key.to_lowercase();

    if matches!(key_lower.as_str(), "home" | "lock" | "power" | "siri") {
        client.button_press(&key_lower).await?;
        return Ok(());
    }

    let key_name = match key_lower.as_str() {
        "enter" | "return" => "return",
        "escape" | "esc" => "escape",
        "delete" | "backspace" => "delete",
        "tab" => "tab",
        "space" => "space",
        "up" => "up",
        "down" => "down",
        "left" => "left",
        "right" => "right",
        _ => {
            return Err(format!(
                "Unknown key: {}. Valid keys: home, lock, siri, enter, tab, space, escape, delete, up, down, left, right",
                key
            )
            .into())
        }
    };

    client.key_press(key_name).await?;
    Ok(())
}
