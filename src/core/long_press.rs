//! long-press コマンド - 長押しジェスチャー
//!
//! ```bash
//! agent-mobile long-press @e1              # ref で長押し
//! agent-mobile long-press "Login"          # テキストで長押し
//! agent-mobile long-press 100,200          # 座標で長押し
//! agent-mobile long-press center           # 画面中央を長押し
//! agent-mobile long-press @e1 --duration 2.0
//! ```

use clap::Args;

use agent_mobile_core::Platform;

use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::ElementTarget;
use super::tap::resolve_coords;
use agent_mobile_gateway::DeviceResolver;

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

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the long-press command
pub async fn run(args: LongPressArgs) -> CommandResult {
    if args.duration <= 0.0 {
        return Err("duration must be greater than 0 seconds".into());
    }

    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;
    let target = ElementTarget::parse(&args.target);

    // Get coordinates from target
    let (x, y) = resolve_coords(&target, platform, args.device.udid.as_deref()).await?;

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
            use agent_mobile_platform_ios::hid::events;

            with_client(udid, |mut client| async move {
                // Passing Some(duration) to tap_to_events inserts a delay between DOWN and UP,
                // turning a regular tap (immediate DOWN+UP) into a long press (DOWN + DELAY + UP).
                let events = events::tap_to_events(x, y, Some(duration));
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::input;

            // Convert duration from seconds to milliseconds
            let duration_ms = (duration * 1000.0) as u64;
            input::long_press(udid, x, y, duration_ms).await?;
            Ok(())
        }
    }
}
