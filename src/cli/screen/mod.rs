//! Screen analysis command implementation.
//!
//! Provides cross-platform screen analysis including accessibility tree dump,
//! screen summary, and navigation hints.

use clap::Args;
use serde::Serialize;

use crate::cli::helpers::{CommandResult, OutputFormat};
use crate::types::Platform;

/// Screen command arguments.
#[derive(Args, Debug)]
pub struct ScreenArgs {
    /// Dump the full accessibility tree.
    #[arg(long)]
    pub dump: bool,

    /// Get a concise screen summary.
    #[arg(long)]
    pub summary: bool,

    /// Get navigation hints (interactive elements).
    #[arg(long)]
    pub hints: bool,

    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,

    /// Output format (human or json).
    #[arg(short = 'o', long, value_enum, default_value = "human")]
    pub output: OutputFormat,
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

/// Execute the screen command.
pub async fn run(args: ScreenArgs) -> CommandResult {
    let platform = match &args.platform {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?,
        None => detect_platform().await?,
    };

    if args.dump {
        return execute_dump(platform, &args).await;
    }

    if args.summary {
        return execute_summary(platform, &args).await;
    }

    if args.hints {
        return execute_hints(platform, &args).await;
    }

    // Default: dump
    execute_dump(platform, &args).await
}

/// Execute accessibility tree dump.
async fn execute_dump(platform: Platform, args: &ScreenArgs) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                let json = client.accessibility_info(None, true).await?;
                println!("{}", json);
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let xml = uiautomator::dump_ui(args.udid.as_deref()).await?;
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;

            if args.output.is_json() {
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
async fn execute_summary(platform: Platform, args: &ScreenArgs) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                let json_str = client.accessibility_info(None, false).await?;

                // Parse the JSON and extract summary
                let json: serde_json::Value = serde_json::from_str(&json_str)?;
                let summary = extract_ios_summary(&json);

                if args.output.is_json() {
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

            let xml = uiautomator::dump_ui(args.udid.as_deref()).await?;
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let summary = extract_android_summary(&elements);

            if args.output.is_json() {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                print_summary(&summary);
            }
            Ok(())
        }
    }
}

/// Execute navigation hints.
async fn execute_hints(platform: Platform, args: &ScreenArgs) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                let json_str = client.accessibility_info(None, true).await?;
                let json: serde_json::Value = serde_json::from_str(&json_str)?;
                let hints = extract_ios_hints(&json);

                if args.output.is_json() {
                    println!("{}", serde_json::to_string_pretty(&hints)?);
                } else {
                    print_hints(&hints);
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let xml = uiautomator::dump_ui(args.udid.as_deref()).await?;
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let hints = extract_android_hints(&elements);

            if args.output.is_json() {
                println!("{}", serde_json::to_string_pretty(&hints)?);
            } else {
                print_hints(&hints);
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

            let role = obj
                .get("AXRole")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            if !label.is_empty() {
                let frame = obj.get("AXFrame").and_then(|f| {
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

    traverse(json, &mut elements);
    elements
}

/// Extract hints (interactive elements) from iOS accessibility JSON.
fn extract_ios_hints(json: &serde_json::Value) -> Vec<ScreenElement> {
    let mut elements = Vec::new();

    fn traverse(node: &serde_json::Value, elements: &mut Vec<ScreenElement>) {
        if let Some(obj) = node.as_object() {
            let label = obj
                .get("AXLabel")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("AXValue").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            let role = obj
                .get("AXRole")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // Filter for interactive elements
            let is_interactive = matches!(
                role.as_str(),
                "AXButton"
                    | "AXLink"
                    | "AXTextField"
                    | "AXSecureTextField"
                    | "AXCell"
                    | "AXMenuItem"
            );

            if !label.is_empty() && is_interactive {
                let frame = obj.get("AXFrame").and_then(|f| {
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
                    clickable: true,
                });
            }

            if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    traverse(child, elements);
                }
            }
        }
    }

    traverse(json, &mut elements);
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

/// Extract hints (interactive elements) from Android elements.
fn extract_android_hints(
    elements: &[crate::platform::android::adb::uiautomator::AccessibilityElement],
) -> Vec<ScreenElement> {
    elements
        .iter()
        .filter_map(|e| {
            if !e.clickable && !e.focused {
                return None;
            }
            let label = e.label().unwrap_or("").to_string();
            if label.is_empty() {
                return None;
            }
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

/// Print hints in human-readable format.
fn print_hints(elements: &[ScreenElement]) {
    println!("Interactive Elements ({} total):", elements.len());
    println!("{:-<60}", "");
    for (i, e) in elements.iter().enumerate() {
        if let Some((x, y)) = e.center {
            println!(
                "{}. [{}] \"{}\" @ ({:.0}, {:.0})",
                i + 1,
                e.element_type,
                e.label,
                x,
                y
            );
        } else {
            println!("{}. [{}] \"{}\"", i + 1, e.element_type, e.label);
        }
    }
}
