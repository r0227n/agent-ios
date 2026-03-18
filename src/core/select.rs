//! select command - Select value from Picker/Spinner
//!
//! ```bash
//! agent-mobile select @e1 "Option 2"       # Select value by Picker/@eN
//! agent-mobile select "Country" "Japan"    # Select value by Picker/text
//! agent-mobile select @e1 "Option 2" --max-swipes 15
//! ```

use std::time::Duration;

use clap::Args;
use serde::Serialize;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::{prepare_xcuitest_with_policy, AppContextPolicy, CommandResult};
use crate::helpers::common_args::DeviceArgs;
use crate::helpers::format::OutputFormat;

use super::ref_resolver::ElementTarget;
use super::swipe::execute_ios_swipe;
use super::tap::{
    execute_ios_tap, execute_tap, resolve_element, resolve_ios_element_with_client,
    take_ios_snapshot_with_client, take_snapshot,
};

/// Arguments for the select command
#[derive(Args, Debug)]
pub struct SelectArgs {
    /// Target picker/spinner: @eN ref or "text"
    pub target: String,

    /// Value to select
    pub value: String,

    /// Maximum number of swipe attempts to find the value
    #[arg(long, default_value = "10")]
    pub max_swipes: u32,

    /// Output format (text or json)
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// JSON output for select command
#[derive(Debug, Serialize)]
struct SelectOutput {
    target: String,
    value: String,
    status: String,
}

/// Execute the select command
pub async fn run(args: SelectArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => run_android(args).await,
    }
}

/// Execute select on iOS (Picker wheel)
async fn run_ios(args: SelectArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);
    let value_lower = args.value.to_lowercase();
    let format = args.format;
    let (resolved_udid, client, _) = prepare_xcuitest_with_policy(
        args.device.udid.as_deref(),
        AppContextPolicy::RestoreIfUnset,
    )
    .await?;

    // 1. Find and tap the picker to activate it
    let picker_element = resolve_ios_element_with_client(&target, &resolved_udid, &client).await?;
    let (picker_x, picker_y) = picker_element.center();

    execute_ios_tap(&client, picker_x, picker_y).await?;

    // Wait for picker to open
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Try to find and select the value by scrolling the picker
    for attempt in 0..args.max_swipes {
        // Take a fresh snapshot
        let snapshot = take_ios_snapshot_with_client(&client).await?;

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
            execute_ios_tap(&client, x, y).await?;

            // Wait a moment then try to dismiss (tap "Done" or outside)
            tokio::time::sleep(Duration::from_millis(300)).await;

            // Try to find and tap "Done" button
            let snapshot = take_ios_snapshot_with_client(&client).await?;
            if let Some(done_btn) = snapshot.elements.iter().find(|e| {
                e.label
                    .as_ref()
                    .map(|l| l == "Done" || l == "完了")
                    .unwrap_or(false)
            }) {
                let (dx, dy) = done_btn.frame.center();
                execute_ios_tap(&client, dx, dy).await?;
            }

            if format.is_json() {
                let output = SelectOutput {
                    target: args.target.clone(),
                    value: args.value.clone(),
                    status: "success".to_string(),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Selected: {}", args.value);
            }
            return Ok(());
        }

        // Not found yet - swipe the picker wheel
        // Alternate between up and down to cover both directions
        let swipe_direction = if attempt % 2 == 0 { -50.0 } else { 50.0 };
        execute_ios_swipe(
            &client,
            picker_x,
            picker_y,
            picker_x,
            picker_y + swipe_direction,
            0.2,
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
    let format = args.format;

    // 1. Find and tap the spinner to open dropdown
    let spinner_element = resolve_element(&target, Platform::Android, udid).await?;
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

            if format.is_json() {
                let output = SelectOutput {
                    target: args.target.clone(),
                    value: args.value.clone(),
                    status: "success".to_string(),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Selected: {}", args.value);
            }
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
