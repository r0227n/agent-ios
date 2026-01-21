//! Keyboard command implementation.
//!
//! Provides cross-platform keyboard input operations including text entry,
//! special keys, and hardware buttons.

use clap::{Args, Subcommand};

use crate::cli::helpers::{CommandResult, DeviceArgs};
use crate::types::Platform;

/// Keyboard command arguments.
#[derive(Args, Debug)]
pub struct KeyboardArgs {
    #[command(subcommand)]
    pub command: KeyboardCommands,
}

/// Keyboard subcommands.
#[derive(Subcommand, Debug)]
pub enum KeyboardCommands {
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

/// Execute the keyboard command.
pub async fn run(args: KeyboardArgs) -> CommandResult {
    match args.command {
        KeyboardCommands::Text { text, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_text(platform, device.udid.as_deref(), &text).await
        }
        KeyboardCommands::Key { key, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_key(platform, device.udid.as_deref(), &key).await
        }
        KeyboardCommands::Button { button, device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_button(platform, device.udid.as_deref(), &button).await
        }
        KeyboardCommands::Clear { device } => {
            let platform = resolve_platform(device.platform.as_deref()).await?;
            execute_clear(platform, device.udid.as_deref()).await
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_keycodes() {
        assert_eq!(ios_keycodes::ENTER, 40);
        assert_eq!(ios_keycodes::BACKSPACE, 42);
    }
}
