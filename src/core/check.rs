//! check/uncheck コマンド - チェックボックス/スイッチの冪等操作
//!
//! ```bash
//! agent-mobile check @e1              # チェックをONにする
//! agent-mobile uncheck @e1            # チェックをOFFにする
//! agent-mobile check "Remember me"   # テキストで指定
//! ```

use clap::Args;

use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::{self, ElementTarget};
use super::tap::{execute_tap, take_snapshot};

/// check/uncheck コマンド引数
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Target: @eN ref or "text"
    pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the check/uncheck command
///
/// - `should_check = true`: チェックをONにする（check コマンド）
/// - `should_check = false`: チェックをOFFにする（uncheck コマンド）
pub async fn run(args: CheckArgs, should_check: bool) -> CommandResult {
    let platform = DeviceResolver::detect_platform().await?;
    let target = ElementTarget::parse(&args.target);

    // Get snapshot and resolve element
    let snapshot = take_snapshot(platform, args.device.udid.as_deref()).await?;
    let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;

    // Check current state from element.value
    // iOS: "1" = checked, "0" = unchecked
    // Android: "true" = checked, "false" = unchecked
    let is_currently_checked = match element.value.as_deref() {
        Some("1") | Some("true") => true,
        Some("0") | Some("false") => false,
        _ => {
            // For elements without value, check traits
            element.traits.iter().any(|t| {
                t.to_lowercase().contains("selected") || t.to_lowercase().contains("checked")
            })
        }
    };

    // Only tap if state needs to change
    if is_currently_checked != should_check {
        let (x, y) = element.center();
        execute_tap(platform, args.device.udid.as_deref(), x, y, None).await?;

        let action = if should_check { "Checked" } else { "Unchecked" };
        println!("{}: {}", action, args.target);
    } else {
        let state = if should_check {
            "already checked"
        } else {
            "already unchecked"
        };
        println!("No action needed: {} is {}", args.target, state);
    }

    Ok(())
}
