//! long-press コマンド - 長押しジェスチャー
//!
//! ```bash
//! agent-mobile long-press @e1              # ref で長押し
//! agent-mobile long-press "Login"          # テキストで長押し
//! agent-mobile long-press 100,200          # 座標で長押し
//! agent-mobile long-press center           # 画面中央を長押し
//! agent-mobile long-press @e1 --duration 2.0
//! agent-mobile long-press @e1 --snapshot snap.json
//! ```

use std::path::PathBuf;

use clap::Args;

use crate::cli::helpers::{with_client, CommandResult, DeviceArgs};
use crate::types::Platform;

use super::ref_resolver::Target;
use super::tap::{resolve_coords, resolve_platform};

/// Default long press duration in seconds
const DEFAULT_LONG_PRESS_DURATION: f64 = 1.0;

/// long-press コマンド引数
#[derive(Args, Debug)]
pub struct LongPressArgs {
    /// Target: @eN ref, "text", x,y coordinates, or position (center)
    pub target: String,

    /// Duration of long press in seconds (default: 1.0)
    #[arg(long, default_value_t = DEFAULT_LONG_PRESS_DURATION)]
    pub duration: f64,

    /// Snapshot file to use (instead of taking fresh snapshot)
    #[arg(long)]
    pub snapshot: Option<PathBuf>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the long-press command
pub async fn run(args: LongPressArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;
    let target = Target::parse(&args.target);

    // Get coordinates from target
    let (x, y) = resolve_coords(
        &target,
        args.snapshot.as_ref(),
        platform,
        args.device.udid.as_deref(),
    )
    .await?;

    // Execute long press
    execute_long_press(platform, args.device.udid.as_deref(), x, y, args.duration).await
}

/// Execute long press gesture
async fn execute_long_press(
    platform: Platform,
    udid: Option<&str>,
    x: f64,
    y: f64,
    duration: f64,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::idb::hid::events;

            with_client(udid, |mut client| async move {
                // tap_to_events with duration creates a long press (DOWN + DELAY + UP)
                let events = events::tap_to_events(x, y, Some(duration));
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::input;

            // Convert duration from seconds to milliseconds
            let duration_ms = (duration * 1000.0) as u64;
            input::long_press(udid, x, y, duration_ms).await?;
            Ok(())
        }
    }
}
