//! fill command - Text field input (clear + type)
//!
//! ```bash
//! agent-mobile fill @e2 "test@example.com"
//! agent-mobile fill "Email" "user@example.com"
//! ```

use clap::Args;

use agent_mobile_core::Platform;

use crate::helpers::client::{with_xcuitest, CommandResult};
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::{self, ElementTarget};
use super::tap::take_snapshot;
use agent_mobile_gateway::DeviceResolver;

/// Arguments for the fill command
#[derive(Args, Debug)]
pub struct FillArgs {
    /// Target text field: @eN ref or "placeholder text"
    pub target: String,

    /// Text to fill
    pub text: String,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the fill command
pub async fn run(args: FillArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };
    let target = ElementTarget::parse(&args.target);

    // Get snapshot and resolve element
    let snapshot = take_snapshot(platform, args.device.udid.as_deref()).await?;

    let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
    let (x, y) = element.center();

    // Execute fill: tap -> clear -> type
    match platform {
        Platform::Ios => execute_fill_ios(x, y, &args.text).await,
        Platform::Android => {
            // Calculate clear length from existing value (Android still uses delete loop)
            let clear_len = element
                .value
                .as_ref()
                .map(|v| v.chars().count())
                .unwrap_or(50);
            execute_fill_android(args.device.udid.as_deref(), x, y, &args.text, clear_len).await
        }
    }
}

/// Execute fill on iOS
async fn execute_fill_ios(x: f64, y: f64, text: &str) -> CommandResult {
    let text = text.to_string();

    with_xcuitest(|client| async move {
        // 1. Tap to focus
        client.tap(x, y).await?;

        // Small delay to ensure focus
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 2. Clear existing text (Select All + Delete)
        client.clear_text().await?;

        // 3. Type new text
        client.type_text(&text).await?;

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
