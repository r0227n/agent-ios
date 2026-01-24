//! get コマンド - 要素情報取得
//!
//! ```bash
//! agent-mobile get text @e1
//! agent-mobile get value @e2
//! agent-mobile get attr @e1 enabled
//! agent-mobile get @e1 -f json      # 全プロパティ (JSON)
//! ```

use clap::Args;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::DeviceArgs;
use crate::helpers::format::OutputFormat;

use super::ref_resolver::{self, ElementTarget};
use super::tap::take_snapshot;
use agent_mobile_gateway::DeviceResolver;

/// get コマンド引数
#[derive(Args, Debug)]
pub struct GetArgs {
    /// Property to get: text, value, attr, box, count, or omit for all
    pub property: String,

    /// Element ref (@eN) or "text"
    pub target: Option<String>,

    /// Attribute name (when property is "attr")
    pub attribute: Option<String>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the get command
pub async fn run(args: GetArgs) -> CommandResult {
    let platform = DeviceResolver::detect_platform().await?;

    // Handle the case where property is actually the target (@eN)
    let (property, target_str) = if args.property.starts_with('@') {
        // get @e1 -> get all properties for @e1
        ("all".to_string(), args.property.clone())
    } else if let Some(t) = args.target.as_ref() {
        (args.property.clone(), t.clone())
    } else {
        return Err(
            "Usage: agent-mobile get <property> <target> or agent-mobile get <target>".into(),
        );
    };

    // Get snapshot
    let snapshot = take_snapshot(platform, args.device.udid.as_deref()).await?;

    // Special handling for 'count' property (doesn't require element resolution)
    if property.to_lowercase() == "count" {
        let target = ElementTarget::parse(&target_str);
        let count = match target {
            ElementTarget::Ref(_) => {
                // Ref exists check
                if ref_resolver::resolve_from_snapshot(&snapshot, &target).is_ok() {
                    1
                } else {
                    0
                }
            }
            ElementTarget::Text(text) => {
                // Count all elements containing text
                let text_lower = text.to_lowercase();
                snapshot
                    .elements
                    .iter()
                    .filter(|e| {
                        e.label
                            .as_ref()
                            .map(|l| l.to_lowercase().contains(&text_lower))
                            .unwrap_or(false)
                            || e.value
                                .as_ref()
                                .map(|v| v.to_lowercase().contains(&text_lower))
                                .unwrap_or(false)
                    })
                    .count()
            }
            _ => {
                return Err("count property requires a ref or text target".into());
            }
        };
        println!("{}", count);
        return Ok(());
    }

    // Resolve element for other properties
    let target = ElementTarget::parse(&target_str);
    let element = ref_resolver::resolve_from_snapshot(&snapshot, &target)?;

    // Get the requested property
    let output = match property.to_lowercase().as_str() {
        "text" | "label" => element.label.clone().unwrap_or_default(),
        "value" => element.value.clone().unwrap_or_default(),
        "type" => element.element_type.clone(),
        "enabled" => element.enabled.to_string(),
        "frame" => {
            format!(
                "x={}, y={}, width={}, height={}",
                element.frame.x, element.frame.y, element.frame.width, element.frame.height
            )
        }
        "box" => {
            if args.format.is_json() {
                serde_json::to_string_pretty(&serde_json::json!({
                    "x": element.frame.x,
                    "y": element.frame.y,
                    "width": element.frame.width,
                    "height": element.frame.height
                }))?
            } else {
                // JSON-style text output for consistency
                format!(
                    r#"{{"x": {}, "y": {}, "width": {}, "height": {}}}"#,
                    element.frame.x, element.frame.y, element.frame.width, element.frame.height
                )
            }
        }
        "center" => {
            let (x, y) = element.center();
            format!("{},{}", x, y)
        }
        "traits" => element.traits.join(", "),
        "attr" => {
            // Get specific attribute
            let attr_name = args
                .attribute
                .as_deref()
                .ok_or("Attribute name required for 'attr' property")?;
            match attr_name.to_lowercase().as_str() {
                "enabled" => element.enabled.to_string(),
                "label" | "text" => element.label.clone().unwrap_or_default(),
                "value" => element.value.clone().unwrap_or_default(),
                "type" => element.element_type.clone(),
                "x" => element.frame.x.to_string(),
                "y" => element.frame.y.to_string(),
                "width" => element.frame.width.to_string(),
                "height" => element.frame.height.to_string(),
                "placeholder" => element.placeholder.clone().unwrap_or_default(),
                "traits" => element.traits.join(", "),
                "depth" => element.depth.to_string(),
                "interactive" | "is_interactive" => element.is_interactive.to_string(),
                _ => return Err(format!("Unknown attribute: {}. Valid attributes: enabled, label, text, value, type, x, y, width, height, placeholder, traits, depth, interactive", attr_name).into()),
            }
        }
        _ if property == "all" || property.starts_with('@') => {
            // Return all properties
            if args.format.is_json() {
                serde_json::to_string_pretty(&serde_json::json!({
                    "ref": element.ref_id,
                    "type": element.element_type,
                    "label": element.label,
                    "value": element.value,
                    "enabled": element.enabled,
                    "frame": {
                        "x": element.frame.x,
                        "y": element.frame.y,
                        "width": element.frame.width,
                        "height": element.frame.height
                    },
                    "traits": element.traits,
                }))?
            } else {
                format!(
                    "ref: {}\ntype: {}\nlabel: {}\nvalue: {}\nenabled: {}\nframe: x={}, y={}, width={}, height={}\ntraits: {}",
                    element.ref_id,
                    element.element_type,
                    element.label.as_deref().unwrap_or("-"),
                    element.value.as_deref().unwrap_or("-"),
                    element.enabled,
                    element.frame.x, element.frame.y, element.frame.width, element.frame.height,
                    if element.traits.is_empty() { "-".to_string() } else { element.traits.join(", ") }
                )
            }
        }
        _ => {
            return Err(format!(
                "Unknown property: {}. Valid properties: text, value, type, enabled, frame, box, center, traits, attr, count, all",
                property
            ).into());
        }
    };

    println!("{}", output);
    Ok(())
}
