//! Screen analysis command implementation.
//!
//! Provides cross-platform screen analysis including accessibility tree dump,
//! screen summary, and navigation hints.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::helpers::{CommandResult, DeviceOutputArgs, OutputFormat};
use crate::types::Platform;

/// Screen command arguments.
#[derive(Args, Debug)]
pub struct ScreenArgs {
    #[command(subcommand)]
    pub command: Option<ScreenCommands>,

    #[command(flatten)]
    pub device_output: DeviceOutputArgs,
}

/// Screen subcommands.
#[derive(Subcommand, Debug)]
pub enum ScreenCommands {
    /// Display the complete accessibility tree of the current screen.
    /// Outputs full UI hierarchy including all nested elements.
    Tree {
        #[command(flatten)]
        device_output: DeviceOutputArgs,
    },

    /// List interactive UI elements with labels and tap coordinates.
    /// Useful for identifying tappable elements and their positions.
    Elements {
        #[command(flatten)]
        device_output: DeviceOutputArgs,
    },
}

/// Simplified screen element for summary/hints.
#[derive(Debug, Serialize)]
pub struct ScreenElement {
    pub label: String,
    pub element_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<(f64, f64)>,
    pub clickable: bool,
}

/// Detect platform based on available devices.
async fn detect_platform() -> Result<Platform, Box<dyn std::error::Error + Send + Sync>> {
    let ios_state_path = std::path::Path::new("/tmp/idb/state");
    if ios_state_path.exists() {
        return Ok(Platform::Ios);
    }

    if crate::platform::android::adb::is_adb_available() {
        let devices = crate::platform::android::adb::list_devices();
        if let Ok(devs) = devices {
            if !devs.is_empty() {
                return Ok(Platform::Android);
            }
        }
    }

    Ok(Platform::Ios)
}

/// Resolve platform from optional string.
async fn resolve_platform(
    platform_str: Option<&str>,
) -> Result<Platform, Box<dyn std::error::Error + Send + Sync>> {
    match platform_str {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() }),
        None => detect_platform().await,
    }
}

/// Execute the screen command.
pub async fn run(args: ScreenArgs) -> CommandResult {
    match args.command {
        Some(ScreenCommands::Tree { device_output }) => {
            let platform = resolve_platform(device_output.platform.as_deref()).await?;
            execute_dump(
                platform,
                device_output.udid.as_deref(),
                &device_output.output,
            )
            .await
        }
        Some(ScreenCommands::Elements { device_output }) => {
            let platform = resolve_platform(device_output.platform.as_deref()).await?;
            execute_summary(
                platform,
                device_output.udid.as_deref(),
                &device_output.output,
            )
            .await
        }
        None => {
            // Default: tree with top-level args
            let platform = resolve_platform(args.device_output.platform.as_deref()).await?;
            execute_dump(
                platform,
                args.device_output.udid.as_deref(),
                &args.device_output.output,
            )
            .await
        }
    }
}

/// Execute accessibility tree dump.
async fn execute_dump(
    platform: Platform,
    udid: Option<&str>,
    output: &OutputFormat,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(udid, |mut client| async move {
                let json = client.accessibility_info(None, true).await?;
                println!("{}", json);
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let xml = uiautomator::dump_ui(udid).await?;
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&elements)?);
            } else {
                // Pretty print the raw XML for human reading
                println!("{}", xml);
            }
            Ok(())
        }
    }
}

/// Execute screen summary.
async fn execute_summary(
    platform: Platform,
    udid: Option<&str>,
    output: &OutputFormat,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            let output = output.clone();
            with_client(udid, |mut client| async move {
                let json_str = client.accessibility_info(None, false).await?;

                // Parse the JSON and extract summary
                let json: serde_json::Value = serde_json::from_str(&json_str)?;
                let summary = extract_ios_summary(&json);

                if output.is_json() {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                } else {
                    print_summary(&summary);
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let xml = uiautomator::dump_ui(udid).await?;
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let summary = extract_android_summary(&elements);

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                print_summary(&summary);
            }
            Ok(())
        }
    }
}

/// Extract summary from iOS accessibility JSON.
fn extract_ios_summary(json: &serde_json::Value) -> Vec<ScreenElement> {
    let mut elements = Vec::new();

    fn traverse(node: &serde_json::Value, elements: &mut Vec<ScreenElement>) {
        if let Some(obj) = node.as_object() {
            let label = obj
                .get("AXLabel")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("AXValue").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            // "role" を使用（"AXRole" ではなく）
            let role = obj
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            if !label.is_empty() {
                // "frame" フィールドを使用（"AXFrame" はテキスト形式のため）
                let frame = obj.get("frame").and_then(|f| {
                    let x = f.get("x")?.as_f64()?;
                    let y = f.get("y")?.as_f64()?;
                    let w = f.get("width")?.as_f64()?;
                    let h = f.get("height")?.as_f64()?;
                    Some((x + w / 2.0, y + h / 2.0))
                });

                elements.push(ScreenElement {
                    label,
                    element_type: role,
                    center: frame,
                    clickable: true, // iOS doesn't explicitly expose this
                });
            }

            if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    traverse(child, elements);
                }
            }
        }
    }

    // トップレベルが配列の場合に対応
    if let Some(arr) = json.as_array() {
        for item in arr {
            traverse(item, &mut elements);
        }
    } else {
        traverse(json, &mut elements);
    }

    elements
}

/// Extract summary from Android elements.
fn extract_android_summary(
    elements: &[crate::platform::android::adb::uiautomator::AccessibilityElement],
) -> Vec<ScreenElement> {
    elements
        .iter()
        .filter_map(|e| {
            let label = e.label()?.to_string();
            Some(ScreenElement {
                label,
                element_type: e.element_type().unwrap_or("Unknown").to_string(),
                center: e.center(),
                clickable: e.clickable,
            })
        })
        .collect()
}

/// Print summary in human-readable format.
fn print_summary(elements: &[ScreenElement]) {
    println!("Screen Elements ({} total):", elements.len());
    println!("{:-<60}", "");
    for e in elements.iter().take(20) {
        if let Some((x, y)) = e.center {
            println!("[{}] {} @ ({:.0}, {:.0})", e.element_type, e.label, x, y);
        } else {
            println!("[{}] {}", e.element_type, e.label);
        }
    }
    if elements.len() > 20 {
        println!("... and {} more elements", elements.len() - 20);
    }
}
