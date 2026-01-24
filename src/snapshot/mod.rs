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

use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::common_args::DeviceArgs;
use crate::helpers::format::OutputFormat;
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

    /// Scope to subtree rooted at element (@eN ref or "text")
    #[arg(short = 's', long)]
    pub scope: Option<String>,

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
    let platform = match args.device.udid.as_deref() {
        Some(udid) => {
            let core_platform = crate::device::detect_platform_from_udid(udid).await?;
            match core_platform {
                agent_mobile_core::Platform::Ios => Platform::Ios,
                agent_mobile_core::Platform::Android => Platform::Android,
            }
        }
        None => resolve_platform().await?,
    };

    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => run_android(args).await,
    }
}

/// Auto-detect platform from connected devices.
async fn resolve_platform() -> CommandResult<Platform> {
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
    let scope = args.scope.clone();
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

        // Apply scope filtering if specified
        let elements = if let Some(scope_target) = &scope {
            extract_subtree(&elements, scope_target)?
        } else {
            elements
        };

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
    let scope = args.scope.clone();
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

    // Apply scope filtering if specified
    let elements = if let Some(scope_target) = &scope {
        extract_subtree(&elements, scope_target)?
    } else {
        elements
    };

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

/// Extract a subtree rooted at the specified element.
///
/// Finds the root element by @eN ref or text, then collects all descendants.
/// Adjusts depth values so the root element has depth 0.
/// Remaps parent_index and children_indices to reflect new array positions.
fn extract_subtree(
    elements: &[types::SnapshotElement],
    scope_target: &str,
) -> CommandResult<Vec<types::SnapshotElement>> {
    use std::collections::HashMap;

    // Find the root element index
    let root_index = if scope_target.starts_with('@') {
        // @eN ref format
        elements
            .iter()
            .position(|e| e.ref_id == scope_target)
            .ok_or_else(|| format!("Element not found: {}", scope_target))?
    } else {
        // Text search using helper method
        elements
            .iter()
            .position(|e| e.contains_text(scope_target))
            .ok_or_else(|| format!("Element with text '{}' not found", scope_target))?
    };

    let root_depth = elements[root_index].depth;

    // Collect indices of elements to include (root and descendants)
    let included_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(idx, _)| is_descendant_or_self(elements, *idx, root_index))
        .map(|(idx, _)| idx)
        .collect();

    if included_indices.is_empty() {
        return Err(format!("No elements found in subtree for: {}", scope_target).into());
    }

    // Create index mapping: old_index -> new_index
    let index_map: HashMap<usize, usize> = included_indices
        .iter()
        .enumerate()
        .map(|(new_idx, &old_idx)| (old_idx, new_idx))
        .collect();

    // Build result with remapped indices
    let result: Vec<types::SnapshotElement> = included_indices
        .iter()
        .map(|&old_idx| {
            let elem = &elements[old_idx];
            let mut elem_clone = elem.clone();

            // Adjust depth relative to root
            elem_clone.depth = elem.depth.saturating_sub(root_depth);

            // Remap parent_index
            elem_clone.parent_index = elem_clone
                .parent_index
                .and_then(|pi| index_map.get(&pi).copied());

            // Remap children_indices
            elem_clone.children_indices = elem_clone
                .children_indices
                .iter()
                .filter_map(|&ci| index_map.get(&ci).copied())
                .collect();

            elem_clone
        })
        .collect();

    Ok(result)
}

/// Check if an element at `idx` is the root or a descendant of element at `root_idx`.
fn is_descendant_or_self(elements: &[types::SnapshotElement], idx: usize, root_idx: usize) -> bool {
    if idx == root_idx {
        return true;
    }

    // Walk up the parent chain
    let mut current = idx;
    while let Some(parent_idx) = elements.get(current).and_then(|e| e.parent_index) {
        if parent_idx == root_idx {
            return true;
        }
        if parent_idx >= current {
            // Prevent infinite loops from malformed data
            break;
        }
        current = parent_idx;
    }

    false
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
}
