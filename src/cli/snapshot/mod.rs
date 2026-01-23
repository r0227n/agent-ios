//! Snapshot command: capture UI accessibility tree with element refs.
//!
//! Provides a snapshot of the current UI state with reference IDs (`@e1`, `@e2`, etc.)
//! for efficient AI agent interaction.
//!
//! Supports both iOS (via idb gRPC) and Android (via ADB/UIAutomator).

mod collector;
pub mod ref_generator;
mod tree_printer;
pub mod types;

use agent_mobile_platform_android::snapshot::extract_android_elements;
use agent_mobile_platform_ios::snapshot::extract_ios_elements;
use chrono::Utc;
use clap::Args;

use crate::cli::helpers::{with_client, CommandResult, DeviceArgs, OutputFormat};
use types::Snapshot;

/// Snapshot command arguments.
#[derive(Args, Debug)]
pub struct SnapshotArgs {
    /// Show only interactive elements (buttons, text fields, etc.)
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// Remove empty structural elements
    #[arg(short = 'c', long)]
    pub compact: bool,

    /// Limit tree depth (e.g., -d 3 shows only top 3 levels)
    #[arg(short = 'd', long)]
    pub depth: Option<u32>,

    /// Output to file instead of stdout
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Output format (text or json). Default: text.
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Disable scrolling (by default, scrolling is enabled to capture all elements)
    #[arg(long)]
    pub no_scroll: bool,

    /// Maximum number of scroll operations (default: 5)
    #[arg(long, default_value = "5")]
    pub max_scrolls: u32,

    /// Delay between scroll operations in milliseconds (default: 500)
    #[arg(long, default_value = "500")]
    pub scroll_delay: u64,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Detected platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Ios,
    Android,
}

/// Execute the snapshot command.
pub async fn run(args: SnapshotArgs) -> CommandResult {
    // Detect or use specified platform
    let platform = resolve_platform(args.device.platform.as_deref()).await?;

    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => run_android(args).await,
    }
}

/// Resolve platform from argument or auto-detect.
async fn resolve_platform(platform_arg: Option<&str>) -> CommandResult<Platform> {
    // If explicitly specified, use that
    if let Some(p) = platform_arg {
        return match p.to_lowercase().as_str() {
            "ios" => Ok(Platform::Ios),
            "android" => Ok(Platform::Android),
            _ => Err(format!("Unknown platform: {}. Use 'ios' or 'android'.", p).into()),
        };
    }

    // Auto-detect: check for iOS companion state first
    if has_ios_companion().await {
        return Ok(Platform::Ios);
    }

    // Then check for Android device
    if has_android_device().await {
        return Ok(Platform::Android);
    }

    Err(
        "No device found. Please connect an iOS simulator/device or Android emulator/device."
            .into(),
    )
}

/// Check if iOS companion is available.
async fn has_ios_companion() -> bool {
    use std::path::Path;

    // Check for companion state file
    let state_path = Path::new("/tmp/idb/state");
    if !state_path.exists() {
        return false;
    }

    // Try to read the state file and check if there are any companions
    if let Ok(content) = std::fs::read_to_string(state_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = json.as_array() {
                return !arr.is_empty();
            }
        }
    }

    false
}

/// Check if Android device is available via ADB.
async fn has_android_device() -> bool {
    use tokio::process::Command;

    let output = Command::new("adb").args(["devices", "-l"]).output().await;

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Check if there's at least one device listed (not just "List of devices attached")
            stdout.lines().skip(1).any(|line| {
                let line = line.trim();
                !line.is_empty() && (line.contains("device") || line.contains("emulator"))
            })
        }
        _ => false,
    }
}

/// Execute snapshot for iOS.
async fn run_ios(args: SnapshotArgs) -> CommandResult {
    let interactive = args.interactive;
    let compact = args.compact;
    let depth = args.depth;
    let output_path = args.output.clone();
    let format = args.format;
    let no_scroll = args.no_scroll;
    let max_scrolls = args.max_scrolls;
    let scroll_delay = args.scroll_delay;

    with_client(args.device.udid.as_deref(), |mut client| async move {
        // Collect raw elements (with or without scrolling)
        let raw_elements = if no_scroll {
            // --no-scroll: Single accessibility info fetch (original behavior)
            let json_str = client.accessibility_info(None, true).await?;
            let json: serde_json::Value = serde_json::from_str(&json_str)?;
            extract_ios_elements(&json)
        } else {
            // Default: Use collector with scrolling to capture all elements
            let config = collector::SnapshotCollectorConfig {
                max_scrolls,
                delay_ms: scroll_delay,
                ..Default::default()
            };
            let snapshot_collector = collector::SnapshotCollector::new(config);
            snapshot_collector.collect_all(&mut client, None).await?
        };

        // Generate refs
        let elements = ref_generator::generate_refs(&raw_elements);

        // Create snapshot
        let snapshot = Snapshot {
            snapshot_id: generate_snapshot_id(),
            timestamp: Utc::now(),
            elements,
        };

        // Format and output
        output_snapshot(
            &snapshot,
            &format,
            interactive,
            compact,
            depth,
            output_path.as_deref(),
        )
    })
    .await
}

/// Execute snapshot for Android.
async fn run_android(args: SnapshotArgs) -> CommandResult {
    use agent_mobile_platform_android::adb::uiautomator;

    let interactive = args.interactive;
    let compact = args.compact;
    let depth = args.depth;
    let output_path = args.output.clone();
    let format = args.format;
    let no_scroll = args.no_scroll;
    let max_scrolls = args.max_scrolls;
    let scroll_delay = args.scroll_delay;
    let serial = args.device.udid.clone();

    // Collect raw elements (with or without scrolling)
    let raw_elements = if no_scroll {
        // --no-scroll: Single UI dump (original behavior)
        let xml = uiautomator::dump_ui(serial.as_deref()).await?;
        let accessibility_elements = uiautomator::parse_ui_hierarchy(&xml)?;
        extract_android_elements(&accessibility_elements)
    } else {
        // Default: Use collector with scrolling to capture all elements
        // Get screen size for scroll calculations
        let (screen_width, screen_height) = collector::get_android_screen_size(serial.as_deref())
            .await
            .unwrap_or((1080.0, 1920.0));

        let config = collector::SnapshotCollectorConfig {
            max_scrolls,
            delay_ms: scroll_delay,
            screen_width,
            screen_height,
        };
        let snapshot_collector = collector::AndroidSnapshotCollector::new(config, serial);
        snapshot_collector.collect_all(None).await?
    };

    // Generate refs
    let elements = ref_generator::generate_refs(&raw_elements);

    // Create snapshot
    let snapshot = Snapshot {
        snapshot_id: generate_snapshot_id(),
        timestamp: Utc::now(),
        elements,
    };

    // Format and output
    output_snapshot(
        &snapshot,
        &format,
        interactive,
        compact,
        depth,
        output_path.as_deref(),
    )
}

/// Format and output the snapshot.
fn output_snapshot(
    snapshot: &Snapshot,
    format: &OutputFormat,
    interactive: bool,
    compact: bool,
    depth: Option<u32>,
    output_path: Option<&str>,
) -> CommandResult {
    let output_str = match format {
        OutputFormat::Text => {
            let options = tree_printer::PrintOptions {
                interactive_only: interactive,
                compact,
                max_depth: depth,
            };
            tree_printer::print_tree(&snapshot.elements, &options)
        }
        OutputFormat::Json => serde_json::to_string_pretty(snapshot)?,
    };

    // Write output
    if let Some(path) = output_path {
        std::fs::write(path, &output_str)?;
        println!("Snapshot saved to: {}", path);
    } else {
        print!("{}", output_str);
    }

    Ok(())
}

/// Generate a unique snapshot ID.
fn generate_snapshot_id() -> String {
    let id = nanoid::nanoid!(8);
    format!("snap_{}", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_snapshot_id() {
        let id1 = generate_snapshot_id();
        let id2 = generate_snapshot_id();

        assert!(id1.starts_with("snap_"));
        assert!(id2.starts_with("snap_"));
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 13); // "snap_" + 8 chars
    }

    #[tokio::test]
    async fn test_resolve_platform_explicit_ios() {
        let result = resolve_platform(Some("ios")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Platform::Ios);
    }

    #[tokio::test]
    async fn test_resolve_platform_explicit_android() {
        let result = resolve_platform(Some("android")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Platform::Android);
    }

    #[tokio::test]
    async fn test_resolve_platform_explicit_unknown() {
        let result = resolve_platform(Some("windows")).await;
        assert!(result.is_err());
    }
}
