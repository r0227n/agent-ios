//! is command - Check element state
//!
//! ```bash
//! agent-mobile is visible @e1
//! agent-mobile is enabled @e1
//! agent-mobile is disabled @e1
//! agent-mobile is interactive @e2
//! ```
//!
//! Outputs "true" or "false" to stdout, and also returns the result
//! via exit code (0 = true, 1 = false).

use clap::Args;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::DeviceArgs;

use super::ref_resolver::ElementTarget;
use super::tap::{query_exists_ios, resolve_element};
use agent_mobile_gateway::DeviceResolver;

/// Arguments for the is command
#[derive(Args, Debug)]
pub struct IsArgs {
    /// State to check: visible, exists, enabled, disabled, interactive, checked
    pub state: String,

    /// Element ref (@eN) or "text"
    pub target: String,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the is command
pub async fn run(args: IsArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    let target = ElementTarget::parse(&args.target);

    // Check the requested state
    let result = match args.state.to_lowercase().as_str() {
        "visible" | "exists" => match (&platform, &target) {
            (agent_mobile_core::Platform::Ios, ElementTarget::Text(text)) => query_exists_ios(
                args.device.udid.as_deref(),
                "text",
                text,
                false,
                false,
                None,
            )
            .await
            .unwrap_or(false),
            _ => resolve_element(&target, platform, args.device.udid.as_deref())
                .await
                .is_ok(),
        },
        "enabled" => match resolve_element(&target, platform, args.device.udid.as_deref()).await {
            Ok(element) => element.enabled,
            Err(_) => false,
        },
        "disabled" => match resolve_element(&target, platform, args.device.udid.as_deref()).await {
            Ok(element) => !element.enabled,
            Err(_) => false,
        },
        "interactive" => {
            match resolve_element(&target, platform, args.device.udid.as_deref()).await {
                Ok(element) => {
                    element.is_text_input() || is_interactive_type(&element.element_type)
                }
                Err(_) => false,
            }
        }
        "checked" => match resolve_element(&target, platform, args.device.udid.as_deref()).await {
            Ok(element) => element
                .value
                .as_ref()
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            Err(_) => false,
        },
        _ => {
            return Err(format!(
                "Unknown state: {}. Valid states: visible, exists, enabled, disabled, interactive, checked",
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
