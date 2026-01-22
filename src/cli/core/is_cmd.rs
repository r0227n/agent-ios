//! is コマンド - 状態確認
//!
//! ```bash
//! agent-mobile is visible @e1
//! agent-mobile is enabled @e1
//! agent-mobile is disabled @e1
//! agent-mobile is interactive @e2
//! ```
//!
//! 結果は stdout に "true" または "false" を出力し、
//! exit code でも結果を返します (0 = true, 1 = false)。

use std::path::PathBuf;

use clap::Args;

use crate::cli::helpers::{CommandResult, DeviceArgs};

use super::ref_resolver::{self, Target};
use super::tap::{resolve_platform, take_snapshot};

/// is コマンド引数
#[derive(Args, Debug)]
pub struct IsArgs {
    /// State to check: visible, exists, enabled, disabled, interactive
    pub state: String,

    /// Element ref (@eN) or "text"
    pub target: String,

    /// Snapshot file to use
    #[arg(long)]
    pub snapshot: Option<PathBuf>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the is command
pub async fn run(args: IsArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;

    // Get snapshot
    let snapshot = if let Some(path) = args.snapshot.as_ref() {
        ref_resolver::load_snapshot_from_file(path)?
    } else {
        take_snapshot(platform, args.device.udid.as_deref()).await?
    };

    let target = Target::parse(&args.target);

    // Check the requested state
    let result = match args.state.to_lowercase().as_str() {
        "visible" | "exists" => {
            // Check if element exists in snapshot
            ref_resolver::resolve_from_snapshot(&snapshot, &target).is_ok()
        }
        "enabled" => {
            // Check if element is enabled
            match ref_resolver::resolve_from_snapshot(&snapshot, &target) {
                Ok(element) => element.enabled,
                Err(_) => false,
            }
        }
        "disabled" => {
            // Check if element is disabled
            match ref_resolver::resolve_from_snapshot(&snapshot, &target) {
                Ok(element) => !element.enabled,
                Err(_) => false,
            }
        }
        "interactive" => {
            // Check if element is interactive
            match ref_resolver::resolve_from_snapshot(&snapshot, &target) {
                Ok(element) => {
                    element.is_text_input() || is_interactive_type(&element.element_type)
                }
                Err(_) => false,
            }
        }
        _ => {
            return Err(format!(
                "Unknown state: {}. Valid states: visible, exists, enabled, disabled, interactive",
                args.state
            )
            .into());
        }
    };

    // Output result
    println!("{}", result);

    // Return error if false (for scripting with exit codes)
    if result {
        Ok(())
    } else {
        // This is a "soft" failure - the command worked but the condition is false
        // We use exit code 1 to indicate false
        std::process::exit(1);
    }
}

/// Check if element type is interactive
fn is_interactive_type(element_type: &str) -> bool {
    matches!(
        element_type,
        "Button"
            | "Link"
            | "TextField"
            | "SecureTextField"
            | "SearchField"
            | "TextArea"
            | "Switch"
            | "Slider"
            | "Stepper"
            | "Picker"
            | "DatePicker"
            | "ImageButton"
            | "FloatingActionButton"
            | "EditText"
            | "CheckBox"
            | "ToggleButton"
    )
}
