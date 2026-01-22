//! get コマンド - 要素情報取得
//!
//! ```bash
//! agent-mobile get text @e1
//! agent-mobile get value @e2
//! agent-mobile get attr @e1 enabled
//! agent-mobile get @e1              # 全プロパティ (JSON)
//! ```

use std::path::PathBuf;

use clap::Args;

use crate::cli::helpers::{CommandResult, DeviceArgs, OutputFormat};

use super::ref_resolver::{self, Target};
use super::tap::{resolve_platform, take_snapshot};

/// get コマンド引数
#[derive(Args, Debug)]
pub struct GetArgs {
    /// Property to get: text, value, attr, or omit for all
    pub property: String,

    /// Element ref (@eN) or "text"
    pub target: Option<String>,

    /// Attribute name (when property is "attr")
    pub attribute: Option<String>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Snapshot file to use
    #[arg(long)]
    pub snapshot: Option<PathBuf>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Execute the get command
pub async fn run(args: GetArgs) -> CommandResult {
    let platform = resolve_platform(args.device.platform.as_deref()).await?;

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

    // Get snapshot and resolve element
    let snapshot = if let Some(path) = args.snapshot.as_ref() {
        ref_resolver::load_snapshot_from_file(path)?
    } else {
        take_snapshot(platform, args.device.udid.as_deref()).await?
    };

    let target = Target::parse(&target_str);
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
                _ => return Err(format!("Unknown attribute: {}", attr_name).into()),
            }
        }
        "all" | _ if property.starts_with('@') || property == "all" => {
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
                "Unknown property: {}. Valid properties: text, value, type, enabled, frame, center, traits, attr, all",
                property
            ).into());
        }
    };

    println!("{}", output);
    Ok(())
}
