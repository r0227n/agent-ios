//! fill コマンド - テキストフィールド入力 (クリア + 入力)
//!
//! ```bash
//! agent-mobile fill @e2 "test@example.com"
//! agent-mobile fill "Email" "user@example.com"
//! ```

use clap::Args;

use agent_mobile_core::Platform;

use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::{self, ElementTarget};
use super::tap::take_snapshot;
use agent_mobile_gateway::DeviceResolver;

/// fill コマンド引数
#[derive(Args, Debug)]
pub struct FillArgs {
    /// Target text field: @eN ref or "placeholder text"
    pub target: String,

    /// Text to fill
    pub text: String,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the fill command
pub async fn run(args: FillArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;
    let target = ElementTarget::parse(&args.target);

    // Get snapshot and resolve element
    let snapshot = take_snapshot(platform, args.device.udid.as_deref()).await?;

    let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
    let (x, y) = element.center();

    // Calculate clear length from existing value
    let clear_len = element
        .value
        .as_ref()
        .map(|v| v.chars().count())
        .unwrap_or(50); // Default to 50 if no value

    // Execute fill: tap -> clear -> type
    match platform {
        Platform::Ios => {
            execute_fill_ios(args.device.udid.as_deref(), x, y, &args.text, clear_len).await
        }
        Platform::Android => {
            execute_fill_android(args.device.udid.as_deref(), x, y, &args.text, clear_len).await
        }
    }
}

/// Execute fill on iOS
async fn execute_fill_ios(
    udid: Option<&str>,
    x: f64,
    y: f64,
    text: &str,
    clear_len: usize,
) -> CommandResult {
    use agent_mobile_platform_ios::hid::events;

    let text = text.to_string();

    with_client(udid, |mut client| async move {
        // 1. Tap to focus
        let tap_events = events::tap_to_events(x, y, None);
        client.hid(tap_events).await?;

        // Small delay to ensure focus
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 2. Clear existing text (based on actual value length)
        for _ in 0..clear_len {
            let del_events = events::key_to_events(42, None); // BACKSPACE = 42
            client.hid(del_events).await?;
        }

        // 3. Type new text
        let text_events = events::text_to_events(&text)?;
        client.hid(text_events).await?;

        Ok(())
    })
    .await
}

/// Execute fill on Android
async fn execute_fill_android(
    udid: Option<&str>,
    x: f64,
    y: f64,
    text: &str,
    clear_len: usize,
) -> CommandResult {
    use agent_mobile_platform_android::adb::input;

    // 1. Tap to focus
    input::tap(udid, x, y).await?;

    // Small delay to ensure focus
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 2. Clear existing text (based on actual value length)
    // Move to end and delete backwards
    input::keyevent_by_name(udid, "KEYCODE_MOVE_END").await?;
    for _ in 0..clear_len {
        input::keyevent(udid, input::keycodes::DEL).await?;
    }

    // 3. Type new text
    input::text(udid, text).await?;

    Ok(())
}
