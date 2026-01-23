//! swipe コマンド - スワイプジェスチャー
//!
//! ```bash
//! agent-mobile swipe up
//! agent-mobile swipe down --from @e3
//! agent-mobile swipe left --distance 300
//! ```

use clap::Args;

use agent_mobile_core::{Platform, ScrollDirection};

use crate::helpers::{with_client, CommandResult, DeviceArgs};

use super::ref_resolver::{self, ElementTarget};
use super::tap::take_snapshot;
use agent_mobile_gateway::DeviceResolver;

/// Default swipe distance in pixels
const DEFAULT_SWIPE_DISTANCE: f64 = 300.0;

/// Default screen dimensions for center calculation
const DEFAULT_SCREEN_WIDTH: f64 = 390.0;
const DEFAULT_SCREEN_HEIGHT: f64 = 844.0;

/// swipe コマンド引数
#[derive(Args, Debug)]
pub struct SwipeArgs {
    /// Direction (up/down/left/right) or coordinates (x1,y1,x2,y2)
    pub direction: String,

    /// Start from element (default: screen center)
    #[arg(long)]
    pub from: Option<String>,

    /// Swipe distance in pixels
    #[arg(long)]
    pub distance: Option<f64>,

    /// Swipe duration in seconds
    #[arg(long)]
    pub duration: Option<f64>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the swipe command
pub async fn run(args: SwipeArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;

    // Parse swipe coordinates
    let ((x1, y1), (x2, y2)) = parse_swipe_args(
        &args.direction,
        args.from.as_deref(),
        args.distance,
        platform,
        args.device.udid.as_deref(),
    )
    .await?;

    // Execute swipe
    match platform {
        Platform::Ios => {
            execute_swipe_ios(args.device.udid.as_deref(), x1, y1, x2, y2, args.duration).await
        }
        Platform::Android => {
            execute_swipe_android(args.device.udid.as_deref(), x1, y1, x2, y2, args.duration).await
        }
    }
}

/// Parse swipe arguments into start and end coordinates
async fn parse_swipe_args(
    direction_arg: &str,
    from_ref: Option<&str>,
    distance: Option<f64>,
    platform: Platform,
    udid: Option<&str>,
) -> CommandResult<((f64, f64), (f64, f64))> {
    let distance = distance.unwrap_or(DEFAULT_SWIPE_DISTANCE);

    // Check if it's a coordinate format (x1,y1,x2,y2)
    if let Some(coords) = parse_swipe_coords(direction_arg) {
        return Ok(coords);
    }

    // Parse as direction
    let direction: ScrollDirection = direction_arg
        .parse()
        .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    // Get start point
    let (cx, cy) = if let Some(from_target) = from_ref {
        // Resolve from ref
        let target = ElementTarget::parse(from_target);
        let snapshot = take_snapshot(platform, udid).await?;
        let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
        element.center()
    } else {
        // Use screen center
        get_screen_center(platform, udid).await?
    };

    // Calculate end point based on direction and distance
    let ((dx1, dy1), (dx2, dy2)) = direction.to_swipe_offsets(distance);
    Ok(((cx + dx1, cy + dy1), (cx + dx2, cy + dy2)))
}

/// Parse swipe coordinates "x1,y1,x2,y2"
fn parse_swipe_coords(s: &str) -> Option<((f64, f64), (f64, f64))> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    let x1: f64 = parts[0].trim().parse().ok()?;
    let y1: f64 = parts[1].trim().parse().ok()?;
    let x2: f64 = parts[2].trim().parse().ok()?;
    let y2: f64 = parts[3].trim().parse().ok()?;
    Some(((x1, y1), (x2, y2)))
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
