//! scroll command - Scroll
//!
//! ```bash
//! agent-mobile scroll down
//! agent-mobile scroll down --in @e5
//! ```

use clap::Args;

use agent_mobile_core::{Platform, ScrollDirection};

use crate::helpers::client::{with_xcuitest, CommandResult};
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::{self, ElementTarget};
use super::tap::take_snapshot;
use agent_mobile_gateway::DeviceResolver;

/// Default scroll distance (shorter than swipe)
const DEFAULT_SCROLL_DISTANCE: f64 = 150.0;

/// Default screen dimensions
const DEFAULT_SCREEN_WIDTH: f64 = 390.0;
const DEFAULT_SCREEN_HEIGHT: f64 = 844.0;

/// Arguments for the scroll command
#[derive(Args, Debug)]
pub struct ScrollArgs {
    /// Direction (up/down/left/right)
    pub direction: String,

    /// Scroll within a specific element
    #[arg(long = "in")]
    pub within: Option<String>,

    /// Scroll distance in pixels
    #[arg(long)]
    pub distance: Option<f64>,

    /// Scroll duration in seconds
    #[arg(long)]
    pub duration: Option<f64>,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the scroll command
pub async fn run(args: ScrollArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    // Parse scroll parameters
    let ((x1, y1), (x2, y2)) = parse_scroll_args(
        &args.direction,
        args.within.as_deref(),
        args.distance,
        platform,
        args.device.udid.as_deref(),
    )
    .await?;

    let duration = args.duration.unwrap_or(0.3);

    // Execute scroll (same as swipe but with shorter distance and duration)
    match platform {
        Platform::Ios => execute_scroll_ios(x1, y1, x2, y2, duration).await,
        Platform::Android => {
            execute_scroll_android(args.device.udid.as_deref(), x1, y1, x2, y2, duration).await
        }
    }
}

/// Parse scroll arguments into start and end coordinates
async fn parse_scroll_args(
    direction_arg: &str,
    within_ref: Option<&str>,
    distance: Option<f64>,
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<((f64, f64), (f64, f64))> {
    let distance = distance.unwrap_or(DEFAULT_SCROLL_DISTANCE);

    // Parse direction
    let direction: ScrollDirection = direction_arg
        .parse()
        .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    // Get scroll center point
    let (cx, cy) = if let Some(within_target) = within_ref {
        // Scroll within a specific element
        let target = ElementTarget::parse(within_target);
        let snapshot = take_snapshot(platform, udid).await?;
        let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
        element.center()
    } else {
        // Scroll at screen center
        get_screen_center(platform, udid).await?
    };

    // Calculate start and end points
    let ((dx1, dy1), (dx2, dy2)) = direction.to_swipe_offsets(distance);
    Ok(((cx + dx1, cy + dy1), (cx + dx2, cy + dy2)))
}

/// Get screen center coordinates
async fn get_screen_center(platform: Platform, udid: Option<&str>) -> CommandResult<(f64, f64)> {
    match platform {
        Platform::Android => {
            match agent_mobile_platform_android::adb::input::get_screen_size(udid).await {
                Ok((w, h)) => Ok((w as f64 / 2.0, h as f64 / 2.0)),
                Err(_) => Ok((DEFAULT_SCREEN_WIDTH / 2.0, DEFAULT_SCREEN_HEIGHT / 2.0)),
            }
        }
        Platform::Ios => Ok((DEFAULT_SCREEN_WIDTH / 2.0, DEFAULT_SCREEN_HEIGHT / 2.0)),
    }
}

/// Execute scroll on iOS
async fn execute_scroll_ios(x1: f64, y1: f64, x2: f64, y2: f64, duration: f64) -> CommandResult {
    with_xcuitest(|client| async move {
        client.swipe((x1, y1), (x2, y2), duration).await?;
        Ok(())
    })
    .await
}

/// Execute scroll on Android
async fn execute_scroll_android(
    udid: Option<&str>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    duration: f64,
) -> CommandResult {
    use agent_mobile_platform_android::adb::input;

    let duration_ms = (duration * 1000.0) as u64;
    input::swipe(udid, x1, y1, x2, y2, Some(duration_ms)).await?;
    Ok(())
}
