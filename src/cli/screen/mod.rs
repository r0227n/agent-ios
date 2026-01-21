//! Screen analysis command implementation.
//!
//! Provides cross-platform screen analysis including accessibility tree dump,
//! screen summary, and navigation hints.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli::helpers::{CommandResult, DeviceOutputArgs, OutputFormat};
use crate::types::Platform;

pub mod collector;
pub mod scrollable;

/// Screen command arguments.
#[derive(Args, Debug)]
pub struct ScreenArgs {
    #[command(subcommand)]
    pub command: Option<ScreenCommands>,

    #[command(flatten)]
    pub device_output: DeviceOutputArgs,

    /// Maximum number of scroll operations (0 = no scrolling).
    #[arg(long, default_value = "3")]
    pub max_scrolls: u32,

    /// Delay between scrolls in milliseconds.
    #[arg(long, default_value = "500")]
    pub delay: u64,
}

/// Collection options for subcommands.
#[derive(Args, Debug, Clone)]
pub struct CollectionArgs {
    /// Maximum number of scroll operations (0 = no scrolling).
    #[arg(long, default_value = "3")]
    pub max_scrolls: u32,

    /// Delay between scrolls in milliseconds.
    #[arg(long, default_value = "500")]
    pub delay: u64,
}

/// Screen subcommands.
#[derive(Subcommand, Debug)]
pub enum ScreenCommands {
    /// List interactive UI elements with labels and tap coordinates.
    /// Useful for identifying tappable elements and their positions.
    Elements {
        #[command(flatten)]
        device_output: DeviceOutputArgs,

        #[command(flatten)]
        collection: CollectionArgs,
    },
}

/// Frame coordinates for an element.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Frame {
    /// Calculate the center point of the frame.
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Simplified screen element for summary/hints.
#[derive(Debug, Clone, Serialize)]
pub struct ScreenElement {
    pub label: String,
    pub element_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<(f64, f64)>,
    pub clickable: bool,
    /// Whether the element is scrollable (inferred from type).
    pub scrollable: bool,
    /// Frame coordinates (for deduplication and position tracking).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<Frame>,
    /// Accessibility identifier (for unique key generation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility_id: Option<String>,
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
        Some(ScreenCommands::Elements {
            device_output,
            collection,
        }) => {
            let platform = resolve_platform(device_output.platform.as_deref()).await?;
            if collection.max_scrolls > 0 {
                execute_summary_all(
                    platform,
                    device_output.udid.as_deref(),
                    &device_output.output,
                    collection.max_scrolls,
                    collection.delay,
                )
                .await
            } else {
                execute_summary(
                    platform,
                    device_output.udid.as_deref(),
                    &device_output.output,
                )
                .await
            }
        }
        None => {
            // Default: accessibility tree with optional scrolling
            let platform = resolve_platform(args.device_output.platform.as_deref()).await?;
            if args.max_scrolls > 0 {
                execute_dump_all(
                    platform,
                    args.device_output.udid.as_deref(),
                    &args.device_output.output,
                    args.max_scrolls,
                    args.delay,
                )
                .await
            } else {
                execute_dump(
                    platform,
                    args.device_output.udid.as_deref(),
                    &args.device_output.output,
                )
                .await
            }
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

/// Execute accessibility tree dump with scrolling.
async fn execute_dump_all(
    platform: Platform,
    udid: Option<&str>,
    output: &OutputFormat,
    max_scrolls: u32,
    delay_ms: u64,
) -> CommandResult {
    use collector::{AndroidCollector, CollectorConfig, IosCollector};

    let config = CollectorConfig {
        max_scrolls,
        delay_ms,
        ..Default::default()
    };

    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            let output = output.clone();
            with_client(udid, |mut client| async move {
                let collector = IosCollector::new(config.clone());

                // Create progress callback for human output
                let progress_fn: Option<collector::ProgressCallback> = if !output.is_json() {
                    let max_scrolls_display = config.max_scrolls;
                    Some(Box::new(move |progress: collector::CollectionProgress| {
                        if progress.scrolling_to_top {
                            eprintln!("Scrolling to top...");
                            return;
                        }
                        if progress.scroll_number == 0 {
                            eprintln!(
                                "Collecting elements (max {} scrolls)...",
                                max_scrolls_display
                            );
                        }
                        if progress.completed {
                            if let Some(reason) = progress.completion_reason {
                                eprintln!(
                                    "Scroll {}: {} elements (+{} new, {})",
                                    progress.scroll_number,
                                    progress.total_elements,
                                    progress.new_elements,
                                    reason
                                );
                            }
                        } else if progress.scroll_number > 0 {
                            eprintln!(
                                "Scroll {}: {} elements (+{} new)",
                                progress.scroll_number,
                                progress.total_elements,
                                progress.new_elements
                            );
                        }
                    }))
                } else {
                    None
                };

                let elements = collector
                    .collect_all(&mut client, extract_ios_summary, progress_fn)
                    .await?;

                if output.is_json() {
                    println!("{}", serde_json::to_string_pretty(&elements)?);
                } else {
                    eprintln!("Total: {} elements", elements.len());
                    print_summary(&elements);
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let max_scrolls_display = config.max_scrolls;
            let collector = AndroidCollector::new(config, udid.map(String::from));

            let progress_fn: Option<collector::ProgressCallback> = if !output.is_json() {
                Some(Box::new(move |progress: collector::CollectionProgress| {
                    if progress.scrolling_to_top {
                        eprintln!("Scrolling to top...");
                        return;
                    }
                    if progress.scroll_number == 0 {
                        eprintln!(
                            "Collecting elements (max {} scrolls)...",
                            max_scrolls_display
                        );
                    }
                    if progress.completed {
                        if let Some(reason) = progress.completion_reason {
                            eprintln!(
                                "Scroll {}: {} elements (+{} new, {})",
                                progress.scroll_number,
                                progress.total_elements,
                                progress.new_elements,
                                reason
                            );
                        }
                    } else if progress.scroll_number > 0 {
                        eprintln!(
                            "Scroll {}: {} elements (+{} new)",
                            progress.scroll_number, progress.total_elements, progress.new_elements
                        );
                    }
                }))
            } else {
                None
            };

            let elements = collector
                .collect_all(
                    |elems: &[uiautomator::AccessibilityElement]| extract_android_summary(elems),
                    progress_fn,
                )
                .await?;

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&elements)?);
            } else {
                eprintln!("Total: {} elements", elements.len());
                print_summary(&elements);
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

/// Execute screen summary with scrolling.
async fn execute_summary_all(
    platform: Platform,
    udid: Option<&str>,
    output: &OutputFormat,
    max_scrolls: u32,
    delay_ms: u64,
) -> CommandResult {
    // Reuse execute_dump_all since it already produces ScreenElements
    execute_dump_all(platform, udid, output, max_scrolls, delay_ms).await
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

            // Extract accessibility identifier
            let accessibility_id = obj
                .get("AXIdentifier")
                .and_then(|v| v.as_str())
                .map(String::from);

            // Extract frame
            let frame = obj.get("frame").and_then(|f| {
                let x = f.get("x")?.as_f64()?;
                let y = f.get("y")?.as_f64()?;
                let w = f.get("width")?.as_f64()?;
                let h = f.get("height")?.as_f64()?;
                Some(Frame {
                    x,
                    y,
                    width: w,
                    height: h,
                })
            });

            let center = frame.as_ref().map(|f| f.center());
            let scrollable = scrollable::is_ios_scrollable(&role);

            if !label.is_empty() {
                elements.push(ScreenElement {
                    label,
                    element_type: role,
                    center,
                    clickable: true, // iOS doesn't explicitly expose this
                    scrollable,
                    frame,
                    accessibility_id,
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
            let element_type = e.element_type().unwrap_or("Unknown").to_string();
            let scrollable = e.scrollable || scrollable::is_android_scrollable(&element_type);

            // Extract frame from bounds
            let frame = e.bounds.map(|[l, t, r, b]| Frame {
                x: l as f64,
                y: t as f64,
                width: (r - l) as f64,
                height: (b - t) as f64,
            });

            Some(ScreenElement {
                label,
                element_type,
                center: e.center(),
                clickable: e.clickable,
                scrollable,
                frame,
                accessibility_id: e.resource_id.clone(),
            })
        })
        .collect()
}

/// Print summary in human-readable format.
fn print_summary(elements: &[ScreenElement]) {
    println!("Screen Elements ({} total):", elements.len());
    println!("{:-<60}", "");
    for e in elements.iter() {
        let scroll_indicator = if e.scrollable { " [scrollable]" } else { "" };
        if let Some((x, y)) = e.center {
            println!(
                "[{}] {} @ ({:.0}, {:.0}){}",
                e.element_type, e.label, x, y, scroll_indicator
            );
        } else {
            println!("[{}] {}{}", e.element_type, e.label, scroll_indicator);
        }
    }
}
