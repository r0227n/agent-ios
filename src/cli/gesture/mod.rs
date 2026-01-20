//! Gesture command implementation.
//!
//! Provides cross-platform touch gesture operations including tap, swipe,
//! scroll, and long-press.

use clap::Args;

use crate::cli::helpers::CommandResult;
use crate::types::{Platform, ScrollDirection};

/// Default screen dimensions for center calculation (iPhone-like).
const DEFAULT_SCREEN_WIDTH: f64 = 390.0;
const DEFAULT_SCREEN_HEIGHT: f64 = 844.0;

/// Default swipe distance in pixels.
const DEFAULT_SWIPE_DISTANCE: f64 = 300.0;

/// Default long press duration in seconds.
const DEFAULT_LONG_PRESS_DURATION: f64 = 1.0;

/// Gesture command arguments.
#[derive(Args, Debug)]
pub struct GestureArgs {
    /// Tap at coordinates (format: "x,y" or "center" for screen center).
    /// If no value given, taps at screen center.
    #[arg(long, num_args = 0..=1, default_missing_value = "center")]
    pub tap: Option<String>,

    /// Swipe gesture (format: "direction" like "up/down/left/right"
    /// or "x1,y1,x2,y2" for custom coordinates).
    #[arg(long)]
    pub swipe: Option<String>,

    /// Scroll in a direction (up/down/left/right).
    /// Similar to swipe but with smaller, controlled movement.
    #[arg(long)]
    pub scroll: Option<String>,

    /// Long press at coordinates (format: "x,y").
    #[arg(long)]
    pub long_press: Option<String>,

    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,

    /// Duration for swipe/scroll/long-press operations (in seconds).
    #[arg(long)]
    pub duration: Option<f64>,
}

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

/// Execute the gesture command.
pub async fn run(args: GestureArgs) -> CommandResult {
    // Determine platform
    let platform = match &args.platform {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?,
        None => detect_platform().await?,
    };

    // Execute the appropriate gesture
    if let Some(ref tap_arg) = args.tap {
        return execute_tap(platform, &args, tap_arg).await;
    }

    if let Some(ref swipe_arg) = args.swipe {
        return execute_swipe(platform, &args, swipe_arg).await;
    }

    if let Some(ref scroll_arg) = args.scroll {
        return execute_scroll(platform, &args, scroll_arg).await;
    }

    if let Some(ref long_press_arg) = args.long_press {
        return execute_long_press(platform, &args, long_press_arg).await;
    }

    Err("No gesture specified. Use --tap, --swipe, --scroll, or --long-press.".into())
}

/// Execute tap gesture.
async fn execute_tap(platform: Platform, args: &GestureArgs, tap_arg: &str) -> CommandResult {
    let (x, y) = if tap_arg == "center" || tap_arg.is_empty() {
        get_screen_center(platform, args.udid.as_deref()).await?
    } else {
        parse_coords(tap_arg)?
    };

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(args.udid.as_deref(), |mut client| async move {
                let events = events::tap_to_events(x, y, args.duration);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            input::tap(args.udid.as_deref(), x, y).await?;
            Ok(())
        }
    }
}

/// Execute swipe gesture.
async fn execute_swipe(platform: Platform, args: &GestureArgs, swipe_arg: &str) -> CommandResult {
    let ((x1, y1), (x2, y2)) = if let Ok(dir) = swipe_arg.parse::<ScrollDirection>() {
        // Direction-based swipe
        let (cx, cy) = get_screen_center(platform, args.udid.as_deref()).await?;
        let ((dx1, dy1), (dx2, dy2)) = dir.to_swipe_offsets(DEFAULT_SWIPE_DISTANCE);
        ((cx + dx1, cy + dy1), (cx + dx2, cy + dy2))
    } else {
        // Coordinate-based swipe
        parse_swipe_coords(swipe_arg)?
    };

    let duration = args.duration;

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(args.udid.as_deref(), |mut client| async move {
                let events = events::swipe_to_events((x1, y1), (x2, y2), duration, None);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            let duration_ms = duration.map(|d| (d * 1000.0) as u64);
            input::swipe(args.udid.as_deref(), x1, y1, x2, y2, duration_ms).await?;
            Ok(())
        }
    }
}

/// Execute scroll gesture.
async fn execute_scroll(platform: Platform, args: &GestureArgs, scroll_arg: &str) -> CommandResult {
    let dir: ScrollDirection = scroll_arg
        .parse()
        .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

    let (cx, cy) = get_screen_center(platform, args.udid.as_deref()).await?;
    // Scroll uses smaller distance than swipe
    let scroll_distance = DEFAULT_SWIPE_DISTANCE * 0.5;
    let ((dx1, dy1), (dx2, dy2)) = dir.to_swipe_offsets(scroll_distance);
    let (x1, y1) = (cx + dx1, cy + dy1);
    let (x2, y2) = (cx + dx2, cy + dy2);

    let duration = args.duration.unwrap_or(0.3);

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(args.udid.as_deref(), |mut client| async move {
                let events = events::swipe_to_events((x1, y1), (x2, y2), Some(duration), None);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            let duration_ms = (duration * 1000.0) as u64;
            input::swipe(args.udid.as_deref(), x1, y1, x2, y2, Some(duration_ms)).await?;
            Ok(())
        }
    }
}

/// Execute long press gesture.
async fn execute_long_press(
    platform: Platform,
    args: &GestureArgs,
    long_press_arg: &str,
) -> CommandResult {
    let (x, y) = parse_coords(long_press_arg)?;
    let duration = args.duration.unwrap_or(DEFAULT_LONG_PRESS_DURATION);

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;
            use crate::cli::idb::hid::events;

            with_client(args.udid.as_deref(), |mut client| async move {
                let events = events::tap_to_events(x, y, Some(duration));
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;
            let duration_ms = (duration * 1000.0) as u64;
            input::long_press(args.udid.as_deref(), x, y, duration_ms).await?;
            Ok(())
        }
    }
}

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
}
