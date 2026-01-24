//! select コマンド - Picker/Spinner から値を選択
//!
//! ```bash
//! agent-mobile select @e1 "Option 2"       # Picker/@eN で値を選択
//! agent-mobile select "Country" "Japan"    # Picker/テキストで値を選択
//! agent-mobile select @e1 "Option 2" --max-swipes 15
//! ```

use std::time::Duration;

use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::{self, ElementTarget};
use super::tap::{execute_tap, take_snapshot};

/// select コマンド引数
#[derive(Args, Debug)]
pub struct SelectArgs {
    /// Target picker/spinner: @eN ref or "text"
    pub target: String,

    /// Value to select
    pub value: String,

    /// Maximum number of swipe attempts to find the value
    #[arg(long, default_value = "10")]
    pub max_swipes: u32,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the select command
pub async fn run(args: SelectArgs) -> CommandResult {
    let platform = DeviceResolver::detect_platform().await?;

    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => run_android(args).await,
    }
}

/// Execute select on iOS (Picker wheel)
async fn run_ios(args: SelectArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);
    let value_lower = args.value.to_lowercase();
    let udid = args.device.udid.as_deref();

    // 1. Find and tap the picker to activate it
    let snapshot = take_snapshot(Platform::Ios, udid).await?;
    let picker_element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
    let (picker_x, picker_y) = picker_element.center();

    execute_tap(Platform::Ios, udid, picker_x, picker_y).await?;

    // Wait for picker to open
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Try to find and select the value by scrolling the picker
    for attempt in 0..args.max_swipes {
        // Take a fresh snapshot
        let snapshot = take_snapshot(Platform::Ios, udid).await?;

        // Look for the value in the current snapshot
        if let Some(element) = snapshot.elements.iter().find(|e| {
            e.label
                .as_ref()
                .map(|l| l.to_lowercase().contains(&value_lower))
                .unwrap_or(false)
                || e.value
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&value_lower))
                    .unwrap_or(false)
        }) {
            // Found it - tap to select
            let (x, y) = element.frame.center();
            execute_tap(Platform::Ios, udid, x, y).await?;

            // Wait a moment then try to dismiss (tap "Done" or outside)
            tokio::time::sleep(Duration::from_millis(300)).await;

            // Try to find and tap "Done" button
            let snapshot = take_snapshot(Platform::Ios, udid).await?;
            if let Some(done_btn) = snapshot.elements.iter().find(|e| {
                e.label
                    .as_ref()
                    .map(|l| l == "Done" || l == "完了")
                    .unwrap_or(false)
            }) {
                let (dx, dy) = done_btn.frame.center();
                execute_tap(Platform::Ios, udid, dx, dy).await?;
            }

            println!("Selected: {}", args.value);
            return Ok(());
        }

        // Not found yet - swipe the picker wheel
        // Alternate between up and down to cover both directions
        let swipe_direction = if attempt % 2 == 0 { -50.0 } else { 50.0 };
        execute_swipe_ios(
            udid,
            picker_x,
            picker_y,
            picker_x,
            picker_y + swipe_direction,
            Some(0.2),
        )
        .await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    Err(format!(
        "Could not find '{}' in picker after {} swipes",
        args.value, args.max_swipes
    )
    .into())
}

/// Execute select on Android (Spinner dropdown)
async fn run_android(args: SelectArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);
    let value_lower = args.value.to_lowercase();
    let udid = args.device.udid.as_deref();

    // 1. Find and tap the spinner to open dropdown
    let snapshot = take_snapshot(Platform::Android, udid).await?;
    let spinner_element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
    let (spinner_x, spinner_y) = spinner_element.center();

    execute_tap(Platform::Android, udid, spinner_x, spinner_y).await?;

    // Wait for dropdown to open
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Find and tap the value in the dropdown
    for attempt in 0..args.max_swipes {
        let snapshot = take_snapshot(Platform::Android, udid).await?;

        // Look for the value in the dropdown list
        if let Some(element) = snapshot.elements.iter().find(|e| {
            e.label
                .as_ref()
                .map(|l| l.to_lowercase().contains(&value_lower))
                .unwrap_or(false)
                || e.value
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&value_lower))
                    .unwrap_or(false)
        }) {
            // Found it - tap to select
            let (x, y) = element.frame.center();
            execute_tap(Platform::Android, udid, x, y).await?;

            println!("Selected: {}", args.value);
            return Ok(());
        }

        // Not found - scroll down in the dropdown
        if attempt < args.max_swipes - 1 {
            // Get screen dimensions for scroll
            let (screen_width, screen_height) =
                agent_mobile_platform_android::adb::input::get_screen_size(udid)
                    .await
                    .unwrap_or((1080, 1920));

            let center_x = screen_width as f64 / 2.0;
            let start_y = screen_height as f64 * 0.6;
            let end_y = screen_height as f64 * 0.4;

            execute_swipe_android(udid, center_x, start_y, center_x, end_y, Some(0.3)).await?;

            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    Err(format!(
        "Could not find '{}' in spinner dropdown after {} scroll attempts",
        args.value, args.max_swipes
    )
    .into())
}

/// Execute swipe on iOS
async fn execute_swipe_ios(
    udid: Option<&str>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    duration: Option<f64>,
) -> CommandResult {
    use agent_mobile_platform_ios::hid::events;

    with_client(udid, |mut client| async move {
        let events = events::swipe_to_events((x1, y1), (x2, y2), duration, None);
        client.hid(events).await?;
        Ok(())
    })
    .await
}

/// Execute swipe on Android
async fn execute_swipe_android(
    udid: Option<&str>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    duration: Option<f64>,
) -> CommandResult {
    use agent_mobile_platform_android::adb::input;

    let duration_ms = duration.map(|d| (d * 1000.0) as u64);
    input::swipe(udid, x1, y1, x2, y2, duration_ms).await?;
    Ok(())
}
