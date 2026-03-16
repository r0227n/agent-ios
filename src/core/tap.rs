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

use agent_mobile_core::{extract_traits_for_type, is_interactive_type, Platform};
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::{
    prepare_xcuitest, prepare_xcuitest_with_policy, with_xcuitest, AppContextPolicy, CommandResult,
};
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::{self, ElementTarget, ResolvedElement};

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
    let target = ElementTarget::parse(&args.target);

    // Handle special keys
    if let ElementTarget::Key(key) = &target {
        return execute_key(platform, args.device.udid.as_deref(), key).await;
    }

    // Get coordinates from target
    let (x, y) = resolve_coords(&target, platform, args.device.udid.as_deref()).await?;

    // Execute tap
    execute_tap(platform, args.device.udid.as_deref(), x, y).await
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
            let element = resolve_element(target, platform, udid).await?;
            Ok(element.center())
        }
        ElementTarget::Key(_) => Err("Cannot resolve key to coordinates".into()),
    }
}

/// Resolve a target into a live element, using the fast iOS query/cache path when available.
pub async fn resolve_element(
    target: &ElementTarget,
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<ResolvedElement> {
    match target {
        ElementTarget::Coords(x, y) => Ok(ResolvedElement::from_coords(*x, *y)),
        ElementTarget::Position(position) => {
            let (x, y) = resolve_position(position, platform, udid).await?;
            Ok(ResolvedElement::from_coords(x, y))
        }
        ElementTarget::Key(_) => Err("Cannot resolve key to an element".into()),
        ElementTarget::Ref(_) | ElementTarget::Text(_) => match platform {
            Platform::Ios => resolve_ios_element(target, udid).await,
            Platform::Android => {
                let snapshot = take_snapshot(platform, udid).await?;
                ref_resolver::resolve_from_snapshot(&snapshot, target)
            }
        },
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
    let (_, client, _) = prepare_xcuitest(udid).await?;
    let snapshot = client
        .snapshot(Some(0), false, false, true, Some(1))
        .await?;
    let frame = snapshot
        .elements
        .first()
        .map(|element| &element.frame)
        .ok_or("Could not determine screen size from snapshot")?;
    Ok((frame.width, frame.height))
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
            let (_, client, _) =
                prepare_xcuitest_with_policy(udid, AppContextPolicy::RestoreIfUnset).await?;
            crate::snapshot::capture_ios_snapshot(&client, None).await
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
                active_bundle_id: None,
                snapshot_generation: None,
                elements,
            })
        }
    }
}

async fn resolve_ios_element(
    target: &ElementTarget,
    udid: Option<&str>,
) -> CommandResult<ResolvedElement> {
    let (resolved_udid, client, _) =
        prepare_xcuitest_with_policy(udid, AppContextPolicy::RestoreIfUnset).await?;

    match target {
        ElementTarget::Text(text) => query_first_ios(&client, "text", text, false, false, None)
            .await?
            .ok_or_else(|| format!("Element with text '{}' not found", text).into()),
        ElementTarget::Ref(ref_id) => {
            let cached_snapshot = crate::snapshot::cache::load_snapshot_cache(&resolved_udid)?;
            let Some(cached_snapshot) = cached_snapshot else {
                return Err(format!(
                    "Element not found: {}\n\nHint: Run 'agent-mobile snapshot' to refresh refs.",
                    ref_id
                )
                .into());
            };

            let cached_element =
                ref_resolver::find_by_ref(&cached_snapshot, ref_id).ok_or_else(|| {
                    format!(
                    "Element not found: {}\n\nHint: Run 'agent-mobile snapshot' to refresh refs.",
                    ref_id
                )
                })?;

            if let Some(element_id) = &cached_element.element_id {
                if let Some(mut resolved) =
                    query_first_ios(&client, "element_id", element_id, true, true, None).await?
                {
                    resolved.ref_id = ref_id.clone();
                    return Ok(resolved);
                }
            } else {
                return Err(format!(
                    "Element not found: {}\n\nHint: Run 'agent-mobile snapshot' to refresh refs.",
                    ref_id
                )
                .into());
            }

            Err(format!(
                "Element not found: {}\n\nHint: Run 'agent-mobile snapshot' to refresh refs.",
                ref_id
            )
            .into())
        }
        _ => Err("Unsupported iOS target resolution".into()),
    }
}

pub(crate) async fn query_first_ios(
    client: &agent_mobile_platform_ios::xcuitest::XCUITestClient,
    locator: &str,
    value: &str,
    exact: bool,
    case_sensitive: bool,
    max_depth: Option<u32>,
) -> CommandResult<Option<ResolvedElement>> {
    let response = client
        .query_first(&agent_mobile_platform_ios::xcuitest::types::QueryRequest {
            locator: locator.to_string(),
            value: value.to_string(),
            exact,
            case_sensitive,
            visible_only: true,
            max_depth,
        })
        .await?;

    Ok(response.element.map(|element| ResolvedElement {
        ref_id: element.element_id.clone(),
        element_id: Some(element.element_id),
        element_type: element.element_type.clone(),
        label: element.label,
        frame: element.frame,
        enabled: element.enabled,
        value: element.value,
        traits: extract_traits_for_type(&element.element_type),
        placeholder: element.placeholder,
        depth: element.depth.unwrap_or(0),
        is_interactive: element.interactive || is_interactive_type(&element.element_type),
    }))
}

pub(crate) async fn query_exists_ios(
    udid: Option<&str>,
    locator: &str,
    value: &str,
    exact: bool,
    case_sensitive: bool,
    max_depth: Option<u32>,
) -> CommandResult<bool> {
    let (_, client, _) =
        prepare_xcuitest_with_policy(udid, AppContextPolicy::RestoreIfUnset).await?;
    let response = client
        .query_exists(&agent_mobile_platform_ios::xcuitest::types::QueryRequest {
            locator: locator.to_string(),
            value: value.to_string(),
            exact,
            case_sensitive,
            visible_only: true,
            max_depth,
        })
        .await?;
    Ok(response.exists)
}

pub(crate) async fn current_ui_hash_ios(
    udid: Option<&str>,
    source: Option<&str>,
    max_depth: Option<u32>,
) -> CommandResult<String> {
    let (_, client, _) =
        prepare_xcuitest_with_policy(udid, AppContextPolicy::RestoreIfUnset).await?;
    Ok(client.ui_hash(source, max_depth, true).await?.hash)
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

/// Execute key press
async fn execute_key(platform: Platform, udid: Option<&str>, key: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            let key_lower = key.to_lowercase();

            // Hardware buttons
            if matches!(key_lower.as_str(), "home" | "lock" | "power" | "siri") {
                let button = key_lower.clone();
                return with_xcuitest(udid, |client| async move {
                    client.button_press(&button).await?;
                    Ok(())
                })
                .await;
            }

            // Keyboard keys
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

            let key_name = key_name.to_string();
            with_xcuitest(udid, |client| async move {
                client.key_press(&key_name).await?;
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
