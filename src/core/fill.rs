//! fill command - Text field input (clear + type)
//!
//! ```bash
//! agent-mobile fill @e2 "test@example.com"
//! agent-mobile fill "Email" "user@example.com"
//! ```

use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_platform_ios::xcuitest::XCUITestClient;

use crate::helpers::client::{with_xcuitest, CommandResult};
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::{self, ElementTarget};
use super::tap::{take_ios_snapshot_with_client, take_snapshot};
use super::text_input::fill_text_input_ios;
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
    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => {
            let target = ElementTarget::parse(&args.target);
            let snapshot = take_snapshot(Platform::Android, args.device.udid.as_deref()).await?;
            let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
            let (x, y) = element.center();

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

async fn run_ios(args: FillArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);

    with_xcuitest(args.device.udid.as_deref(), |client| async move {
        let snapshot = take_ios_snapshot_with_client(&client).await?;
        let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
        let (x, y) = element.center();
        execute_fill_ios_with_client(&client, x, y, &args.text).await
    })
    .await
}

pub(crate) async fn execute_fill_ios_with_client(
    client: &XCUITestClient,
    x: f64,
    y: f64,
    text: &str,
) -> CommandResult {
    fill_text_input_ios(client, x, y, text).await
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
