//! fill コマンド - テキストフィールド入力 (クリア + 入力)
//!
//! ```bash
//! agent-mobile fill @e2 "test@example.com"
//! agent-mobile fill "Email" "user@example.com"
//! ```

use std::path::PathBuf;

use clap::Args;

use crate::cli::helpers::{with_client, CommandResult, DeviceArgs};
use crate::types::Platform;

use super::ref_resolver::{self, Target};
use super::tap::{resolve_platform, take_snapshot};

/// fill コマンド引数
#[derive(Args, Debug)]
pub struct FillArgs {
    /// Target text field: @eN ref or "placeholder text"
    pub target: String,

    /// Text to fill
    pub text: String,

    /// Snapshot file to use (instead of taking fresh snapshot)
    #[arg(long)]
    pub snapshot: Option<PathBuf>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the fill command
pub async fn run(args: FillArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;
    let target = Target::parse(&args.target);

    // Get snapshot and resolve element
    let snapshot = if let Some(path) = args.snapshot.as_ref() {
        ref_resolver::load_snapshot_from_file(path)?
    } else {
        take_snapshot(platform, args.device.udid.as_deref()).await?
    };

    let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;
    let (x, y) = element.center();

    // Execute fill: tap -> clear -> type
    match platform {
        Platform::Ios => execute_fill_ios(args.device.udid.as_deref(), x, y, &args.text).await,
        Platform::Android => {
            execute_fill_android(args.device.udid.as_deref(), x, y, &args.text).await
        }
    }
}

/// Execute fill on iOS
async fn execute_fill_ios(udid: Option<&str>, x: f64, y: f64, text: &str) -> CommandResult {
    use crate::cli::idb::hid::events;

    let text = text.to_string();

    with_client(udid, |mut client| async move {
        // 1. Tap to focus
        let tap_events = events::tap_to_events(x, y, None);
        client.hid(tap_events).await?;

        // Small delay to ensure focus
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // 2. Clear existing text (multiple backspaces)
        // TODO: Implement proper select all + delete
        for _ in 0..50 {
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
async fn execute_fill_android(udid: Option<&str>, x: f64, y: f64, text: &str) -> CommandResult {
    use crate::platform::android::adb::input;

    // 1. Tap to focus
    input::tap(udid, x, y).await?;

    // Small delay to ensure focus
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 2. Clear existing text
    // Move to end and delete backwards
    input::keyevent_by_name(udid, "KEYCODE_MOVE_END").await?;
    for _ in 0..50 {
        input::keyevent(udid, input::keycodes::DEL).await?;
    }

    // 3. Type new text
    input::text(udid, text).await?;

    Ok(())
}
