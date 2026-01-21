//! HID (Human Interface Device) command implementation.
//!
//! Provides cross-platform HID operations including touch gestures (tap, swipe,
//! scroll, long-press) and keyboard input (text, keys, buttons, clear).

use clap::{Args, Subcommand};

use crate::cli::helpers::{CommandResult, DeviceArgs};
use crate::types::{Platform, ScrollDirection};

/// Default screen dimensions for center calculation (iPhone-like).
const DEFAULT_SCREEN_WIDTH: f64 = 390.0;
const DEFAULT_SCREEN_HEIGHT: f64 = 844.0;

/// Default swipe distance in pixels.
const DEFAULT_SWIPE_DISTANCE: f64 = 300.0;

/// Default long press duration in seconds.
const DEFAULT_LONG_PRESS_DURATION: f64 = 1.0;

/// HID command arguments.
#[derive(Args, Debug)]
pub struct HidArgs {
    #[command(subcommand)]
    pub command: HidCommands,
}

/// HID subcommands (gestures + keyboard).
#[derive(Subcommand, Debug)]
pub enum HidCommands {
    // ========== Touch Gestures ==========
    /// Tap at coordinates.
    Tap {
        /// Coordinates (format: "x,y" or "center" for screen center).
        #[arg(default_value = "center")]
        coordinates: String,

        #[command(flatten)]
        device: DeviceArgs,

        /// Duration of the tap in seconds.
        #[arg(long)]
        duration: Option<f64>,
    },

    /// Swipe gesture.
    Swipe {
        /// Direction (up/down/left/right) or custom coordinates (x1,y1,x2,y2).
        direction: String,

        #[command(flatten)]
        device: DeviceArgs,

        /// Duration of the swipe in seconds.
        #[arg(long)]
        duration: Option<f64>,
    },

    /// Scroll in a direction.
    Scroll {
        /// Direction (up/down/left/right).
        direction: String,

        #[command(flatten)]
        device: DeviceArgs,

        /// Duration of the scroll in seconds.
        #[arg(long)]
        duration: Option<f64>,
    },

    /// Long press at coordinates.
    #[command(name = "long-press")]
    LongPress {
        /// Coordinates (format: "x,y").
        coordinates: String,

        #[command(flatten)]
        device: DeviceArgs,

        /// Duration of the long press in seconds.
        #[arg(long)]
        duration: Option<f64>,
    },

    // ========== Keyboard Input ==========
    /// Input text string.
    Text {
        /// Text to input.
        text: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Press a special key (enter, delete, tab, space, escape, up, down, left, right).
    Key {
        /// Key name (enter, delete, tab, space, escape, up, down, left, right).
        key: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Press a hardware button (home, lock, volume-up, volume-down, back, menu).
    Button {
        /// Button name (home, lock, volume-up, volume-down, back, menu).
        button: String,

        #[command(flatten)]
        device: DeviceArgs,
    },

    /// Clear current text input (select all + delete).
    Clear {
        #[command(flatten)]
        device: DeviceArgs,
    },
}

// ==================== Helper Functions ====================

/// Parse coordinate string like "100,200" into (x, y).
fn parse_coords(s: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid coordinate format '{}'. Expected 'x,y'.",
            s
        ));
    }
    let x: f64 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid x coordinate: {}", parts[0]))?;
    let y: f64 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid y coordinate: {}", parts[1]))?;
    Ok((x, y))
}

/// Parse swipe coordinates "x1,y1,x2,y2" into start and end points.
fn parse_swipe_coords(s: &str) -> Result<((f64, f64), (f64, f64)), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return Err(format!(
            "Invalid swipe format '{}'. Expected 'x1,y1,x2,y2'.",
            s
        ));
    }
    let x1: f64 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid x1 coordinate: {}", parts[0]))?;
    let y1: f64 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid y1 coordinate: {}", parts[1]))?;
    let x2: f64 = parts[2]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid x2 coordinate: {}", parts[2]))?;
    let y2: f64 = parts[3]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid y2 coordinate: {}", parts[3]))?;
    Ok(((x1, y1), (x2, y2)))
}

/// Detect platform based on available devices.
async fn detect_platform() -> Result<Platform, Box<dyn std::error::Error + Send + Sync>> {
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

/// Get screen center coordinates.
async fn get_screen_center(
    platform: Platform,
    serial: Option<&str>,
) -> Result<(f64, f64), Box<dyn std::error::Error + Send + Sync>> {
    match platform {
        Platform::Android => {
            match crate::platform::android::adb::input::get_screen_size(serial).await {
                Ok((w, h)) => Ok((w as f64 / 2.0, h as f64 / 2.0)),
                Err(_) => Ok((DEFAULT_SCREEN_WIDTH / 2.0, DEFAULT_SCREEN_HEIGHT / 2.0)),
            }
        }
        Platform::Ios => {
            // iOS: Use default dimensions for now
            // TODO: Get actual screen size from device description
            Ok((DEFAULT_SCREEN_WIDTH / 2.0, DEFAULT_SCREEN_HEIGHT / 2.0))
        }
    }
}

/// iOS HID keycodes for special keys.
mod ios_keycodes {
    pub const ENTER: u64 = 40;
    pub const ESCAPE: u64 = 41;
    pub const BACKSPACE: u64 = 42;
    pub const TAB: u64 = 43;
    pub const SPACE: u64 = 44;
    #[allow(dead_code)]
    pub const DELETE: u64 = 76;
    pub const UP: u64 = 82;
    pub const DOWN: u64 = 81;
    pub const LEFT: u64 = 80;
    pub const RIGHT: u64 = 79;
}

// ==================== Main Entry Point ====================

/// Execute the HID command.
pub async fn run(args: HidArgs) -> CommandResult {
    match args.command {
        // Touch Gestures
        HidCommands::Tap {
            coordinates,
            device,
            duration,
        } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_tap(platform, device.udid.as_deref(), &coordinates, duration).await
        }
        HidCommands::Swipe {
            direction,
            device,
            duration,
        } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_swipe(platform, device.udid.as_deref(), &direction, duration).await
        }
        HidCommands::Scroll {
            direction,
            device,
            duration,
        } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_scroll(platform, device.udid.as_deref(), &direction, duration).await
        }
        HidCommands::LongPress {
            coordinates,
            device,
            duration,
        } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_long_press(platform, device.udid.as_deref(), &coordinates, duration).await
        }

        // Keyboard Input
        HidCommands::Text { text, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_text(platform, device.udid.as_deref(), &text).await
        }
        HidCommands::Key { key, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_key(platform, device.udid.as_deref(), &key).await
        }
        HidCommands::Button { button, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_button(platform, device.udid.as_deref(), &button).await
        }
        HidCommands::Clear { device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_clear(platform, device.udid.as_deref()).await
        }
    }
}

// ==================== Touch Gesture Implementations ====================

/// Execute tap gesture.
async fn execute_tap(
    platform: Platform,
    udid: Option<&str>,
    coordinates: &str,
    duration: Option<f64>,
) -> CommandResult {
    let (x, y) = if coordinates == "center" || coordinates.is_empty() {
        get_screen_center(platform, udid).await?
    } else {
        parse_coords(coordinates)?
    };

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
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

/// Execute swipe gesture.
async fn execute_swipe(
    platform: Platform,
    udid: Option<&str>,
    direction_arg: &str,
    duration: Option<f64>,
) -> CommandResult {
    let ((x1, y1), (x2, y2)) = if let Ok(dir) = direction_arg.parse::<ScrollDirection>() {
        // Direction-based swipe
        let (cx, cy) = get_screen_center(platform, udid).await?;
        let ((dx1, dy1), (dx2, dy2)) = dir.to_swipe_offsets(DEFAULT_SWIPE_DISTANCE);
        ((cx + dx1, cy + dy1), (cx + dx2, cy + dy2))
    } else {
        // Coordinate-based swipe
        parse_swipe_coords(direction_arg)?
    };

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(udid, |mut client| async move {
                let events = events::swipe_to_events((x1, y1), (x2, y2), duration, None);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            let duration_ms = duration.map(|d| (d * 1000.0) as u64);
            input::swipe(udid, x1, y1, x2, y2, duration_ms).await?;
            Ok(())
        }
    }
}

/// Execute scroll gesture.
async fn execute_scroll(
    platform: Platform,
    udid: Option<&str>,
    direction_arg: &str,
    duration: Option<f64>,
) -> CommandResult {
    let dir: ScrollDirection = direction_arg
        .parse()
        .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    let (cx, cy) = get_screen_center(platform, udid).await?;
    // Scroll uses smaller distance than swipe
    let scroll_distance = DEFAULT_SWIPE_DISTANCE * 0.5;
    let ((dx1, dy1), (dx2, dy2)) = dir.to_swipe_offsets(scroll_distance);
    let (x1, y1) = (cx + dx1, cy + dy1);
    let (x2, y2) = (cx + dx2, cy + dy2);

    let duration = duration.unwrap_or(0.3);

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(udid, |mut client| async move {
                let events = events::swipe_to_events((x1, y1), (x2, y2), Some(duration), None);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            let duration_ms = (duration * 1000.0) as u64;
            input::swipe(udid, x1, y1, x2, y2, Some(duration_ms)).await?;
            Ok(())
        }
    }
}

/// Execute long press gesture.
async fn execute_long_press(
    platform: Platform,
    udid: Option<&str>,
    coordinates: &str,
    duration: Option<f64>,
) -> CommandResult {
    let (x, y) = parse_coords(coordinates)?;
    let duration = duration.unwrap_or(DEFAULT_LONG_PRESS_DURATION);

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(udid, |mut client| async move {
                let events = events::tap_to_events(x, y, Some(duration));
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            let duration_ms = (duration * 1000.0) as u64;
            input::long_press(udid, x, y, duration_ms).await?;
            Ok(())
        }
    }
}

// ==================== Keyboard Input Implementations ====================

/// Execute text input.
async fn execute_text(platform: Platform, udid: Option<&str>, text: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            let text = text.to_string();

            with_client(udid, |mut client| async move {
                let events = events::text_to_events(&text)?;
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            input::text(udid, text).await?;
            Ok(())
        }
    }
}

/// Execute special key press.
async fn execute_key(platform: Platform, udid: Option<&str>, key: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            let keycode = match key.to_lowercase().as_str() {
                "enter" | "return" => ios_keycodes::ENTER,
                "delete" | "backspace" => ios_keycodes::BACKSPACE,
                "tab" => ios_keycodes::TAB,
                "space" => ios_keycodes::SPACE,
                "escape" | "esc" => ios_keycodes::ESCAPE,
                "up" => ios_keycodes::UP,
                "down" => ios_keycodes::DOWN,
                "left" => ios_keycodes::LEFT,
                "right" => ios_keycodes::RIGHT,
                _ => {
                    return Err(format!(
                        "Unknown key: {}. Valid keys: enter, delete, tab, space, escape, up, down, left, right",
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

            let keycode = match key.to_lowercase().as_str() {
                "enter" | "return" => keycodes::ENTER,
                "delete" | "backspace" => keycodes::DEL,
                "tab" => keycodes::TAB,
                "space" => keycodes::SPACE,
                "escape" | "esc" => keycodes::ESCAPE,
                "up" => keycodes::DPAD_UP,
                "down" => keycodes::DPAD_DOWN,
                "left" => keycodes::DPAD_LEFT,
                "right" => keycodes::DPAD_RIGHT,
                _ => {
                    return Err(format!(
                        "Unknown key: {}. Valid keys: enter, delete, tab, space, escape, up, down, left, right",
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

/// Execute hardware button press.
async fn execute_button(platform: Platform, udid: Option<&str>, button: &str) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;
            use crate::grpc::idb::hid_event::HidButtonType;

            let button_type = match button.to_lowercase().as_str() {
                "home" => HidButtonType::Home,
                "lock" | "power" => HidButtonType::Lock,
                "side" | "side_button" => HidButtonType::SideButton,
                "siri" => HidButtonType::Siri,
                "apple_pay" => HidButtonType::ApplePay,
                _ => {
                    return Err(format!(
                        "Unknown iOS button: {}. Valid buttons: home, lock, side, siri, apple_pay",
                        button
                    )
                    .into())
                }
            };

            with_client(udid, |mut client| async move {
                let events = events::button_to_events(button_type, None);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input::{self, keycodes};

            let keycode = match button.to_lowercase().as_str() {
                "home" => keycodes::HOME,
                "back" => keycodes::BACK,
                "menu" => keycodes::MENU,
                "power" | "lock" => keycodes::POWER,
                "volume-up" | "volume_up" => keycodes::VOLUME_UP,
                "volume-down" | "volume_down" => keycodes::VOLUME_DOWN,
                "app_switch" | "recent" | "recents" => keycodes::APP_SWITCH,
                _ => {
                    return Err(format!(
                        "Unknown Android button: {}. Valid buttons: home, back, menu, power, volume-up, volume-down, app_switch",
                        button
                    )
                    .into())
                }
            };

            input::keyevent(udid, keycode).await?;
            Ok(())
        }
    }
}

/// Execute clear (select all + delete).
async fn execute_clear(platform: Platform, udid: Option<&str>) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(udid, |mut client| async move {
                // Cmd+A (select all) - keycode 4 (A) with modifier 8 (Cmd)
                // For now, send multiple backspaces as a simple clear
                // TODO: Implement proper select all + delete
                for _ in 0..50 {
                    let events = events::key_to_events(ios_keycodes::BACKSPACE, None);
                    client.hid(events).await?;
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;

            // Android: Move to end and delete backwards
            // First move to end of text field
            input::keyevent_by_name(udid, "KEYCODE_MOVE_END").await?;

            // Then delete 50 characters (reasonable max for most fields)
            for _ in 0..50 {
                input::keyevent(udid, input::keycodes::DEL).await?;
            }
            Ok(())
        }
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coords() {
        assert_eq!(parse_coords("100,200").unwrap(), (100.0, 200.0));
        assert_eq!(parse_coords("100.5,200.5").unwrap(), (100.5, 200.5));
        assert!(parse_coords("invalid").is_err());
        assert!(parse_coords("100").is_err());
    }

    #[test]
    fn test_parse_swipe_coords() {
        let ((x1, y1), (x2, y2)) = parse_swipe_coords("100,200,300,400").unwrap();
        assert_eq!((x1, y1), (100.0, 200.0));
        assert_eq!((x2, y2), (300.0, 400.0));
        assert!(parse_swipe_coords("100,200").is_err());
    }

    #[test]
    fn test_ios_keycodes() {
        assert_eq!(ios_keycodes::ENTER, 40);
        assert_eq!(ios_keycodes::BACKSPACE, 42);
    }
}
