//! Snapshot command: capture UI accessibility tree with element refs.
//!
//! Provides a snapshot of the current UI state with reference IDs (`@e1`, `@e2`, etc.)
//! for efficient AI agent interaction.
//!
//! Supports both iOS (via XCUITest Runner) and Android (via ADB/UIAutomator).

pub mod cache;
mod collector;
pub mod ref_generator;
mod tree_printer;
pub mod types;

use agent_mobile_core::{extract_traits_for_type, is_interactive_type};
use agent_mobile_platform_android::snapshot::extract_android_elements;
use agent_mobile_platform_ios::xcuitest::types::{RunnerSnapshotElement, RunnerSnapshotResponse};
use chrono::Utc;
use clap::Args;

use crate::helpers::client::{prepare_xcuitest_with_policy, AppContextPolicy, CommandResult};
use crate::helpers::common_args::DeviceArgs;
use crate::helpers::format::OutputFormat;
use crate::helpers::ios::get_ios_screen_size_from_client;
use types::Snapshot;

const IOS_SNAPSHOT_MAX_NODES: u32 = 512;

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

    /// Maximum number of scroll operations (default: 3)
    #[arg(long, default_value = "3")]
    pub max_scrolls: u32,

    /// Delay between scroll operations in milliseconds (default: 100)
    #[arg(long, default_value = "100")]
    pub scroll_delay: u64,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Detected platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Capture a snapshot from an iOS device or simulator.
    Ios,
    /// Capture a snapshot from an Android device or emulator.
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
    // Auto-detect: check for booted iOS simulator first
    if has_ios_simulator().await {
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

/// Check if a booted iOS simulator is available.
async fn has_ios_simulator() -> bool {
    agent_mobile_platform_ios::simctl::get_booted_simulator().is_ok()
}

/// Check if Android device is available via native ADB protocol.
async fn has_android_device() -> bool {
    if !agent_mobile_platform_android::is_adb_available() {
        return false;
    }
    match agent_mobile_platform_android::list_devices() {
        Ok(devices) => devices.iter().any(|(_, state)| state == "device"),
        Err(_) => false,
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

    let (resolved_udid, client, _) = prepare_xcuitest_with_policy(
        args.device.udid.as_deref(),
        AppContextPolicy::RestoreIfUnset,
    )
    .await?;

    let snapshot = if no_scroll {
        capture_ios_snapshot(&client, depth).await?
    } else {
        // Default: Use collector with scrolling to capture visible elements from multiple positions.
        let (screen_width, screen_height) = get_ios_screen_size_from_client(&client)
            .await
            .unwrap_or((390.0, 844.0));
        let config = collector::SnapshotCollectorConfig {
            max_scrolls,
            delay_ms: scroll_delay,
            screen_width,
            screen_height,
        };
        let snapshot_collector = collector::SnapshotCollector::new(config);
        snapshot_collector.collect_snapshot(&client, None).await?
    };

    let snapshot = finalize_snapshot(snapshot, interactive, compact, depth, scope.as_deref())?;
    cache::save_snapshot_cache(&resolved_udid, &snapshot)?;

    output_snapshot(&snapshot, &format, output_path.as_deref())
}

pub(crate) async fn capture_ios_snapshot(
    client: &agent_mobile_platform_ios::xcuitest::XCUITestClient,
    depth: Option<u32>,
) -> CommandResult<Snapshot> {
    let response = client
        .snapshot(depth, false, false, true, Some(IOS_SNAPSHOT_MAX_NODES))
        .await?;
    Ok(snapshot_from_runner_response(response))
}

fn snapshot_from_runner_response(response: RunnerSnapshotResponse) -> Snapshot {
    let RunnerSnapshotResponse {
        snapshot_id,
        active_bundle_id,
        snapshot_generation,
        elements: runner_elements,
    } = response;

    let mut elements: Vec<types::SnapshotElement> = runner_elements
        .iter()
        .map(|element| types::SnapshotElement {
            ref_id: String::new(),
            element_id: Some(element.element_id.clone()),
            element_type: element.element_type.clone(),
            label: element.label.clone(),
            frame: element.frame.clone(),
            enabled: element.enabled,
            traits: extract_traits_for_type(&element.element_type),
            placeholder: element.placeholder.clone(),
            value: element.value.clone(),
            children_indices: Vec::new(),
            depth: element.depth.unwrap_or(0),
            is_interactive: element.interactive || is_interactive_type(&element.element_type),
            parent_index: None,
        })
        .collect();

    if runner_elements
        .iter()
        .all(|element| element.depth.is_some())
    {
        apply_explicit_hierarchy(&mut elements, &runner_elements);
    } else {
        infer_hierarchy_from_frames(&mut elements);
    }

    ref_generator::assign_refs(&mut elements);

    Snapshot {
        snapshot_id,
        timestamp: Utc::now(),
        active_bundle_id,
        snapshot_generation: Some(snapshot_generation),
        elements,
    }
}

fn apply_explicit_hierarchy(
    elements: &mut [types::SnapshotElement],
    runner_elements: &[RunnerSnapshotElement],
) {
    let mut stack: Vec<usize> = Vec::new();

    for (index, runner) in runner_elements.iter().enumerate() {
        let depth = runner.depth.unwrap_or(0) as usize;
        elements[index].depth = depth as u32;

        while stack.len() > depth {
            stack.pop();
        }

        if depth > 0 {
            if let Some(&parent_index) = stack.last() {
                elements[index].parent_index = Some(parent_index);
                elements[parent_index].children_indices.push(index);
            }
        } else {
            elements[index].parent_index = None;
        }

        stack.push(index);
    }
}

pub(crate) fn infer_hierarchy_from_frames(elements: &mut [types::SnapshotElement]) {
    if elements.is_empty() {
        return;
    }

    elements[0].depth = 0;
    elements[0].parent_index = None;

    for index in 1..elements.len() {
        let child_frame = elements[index].frame.clone();
        let mut best_parent = 0usize;
        let mut best_area = f64::INFINITY;

        for (candidate, parent) in elements.iter().enumerate().take(index) {
            let parent_frame = &parent.frame;
            if !frame_contains(parent_frame, &child_frame) {
                continue;
            }

            let area = parent_frame.width.max(1.0) * parent_frame.height.max(1.0);
            if area < best_area {
                best_area = area;
                best_parent = candidate;
            }
        }

        elements[index].parent_index = Some(best_parent);
        elements[index].depth = elements[best_parent].depth + 1;
        elements[best_parent].children_indices.push(index);
    }
}

fn frame_contains(
    parent: &agent_mobile_core::snapshot::Frame,
    child: &agent_mobile_core::snapshot::Frame,
) -> bool {
    let epsilon = 1.0;
    parent.x - epsilon <= child.x
        && parent.y - epsilon <= child.y
        && parent.x + parent.width + epsilon >= child.x + child.width
        && parent.y + parent.height + epsilon >= child.y + child.height
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

    // Create snapshot
    let snapshot = Snapshot {
        snapshot_id: generate_snapshot_id(),
        timestamp: Utc::now(),
        active_bundle_id: None,
        snapshot_generation: None,
        elements,
    };

    let snapshot = finalize_snapshot(snapshot, interactive, compact, depth, scope.as_deref())?;
    output_snapshot(&snapshot, &format, output_path.as_deref())
}

/// Format and output the snapshot.
fn output_snapshot(
    snapshot: &Snapshot,
    format: &OutputFormat,
    output_path: Option<&str>,
) -> CommandResult {
    let output_str = match format {
        OutputFormat::Text => {
            // finalize_snapshot already applied CLI filters to snapshot.elements.
            let options = tree_printer::PrintOptions {
                interactive_only: false,
                compact: false,
                max_depth: None,
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

fn finalize_snapshot(
    snapshot: Snapshot,
    interactive: bool,
    compact: bool,
    depth: Option<u32>,
    scope: Option<&str>,
) -> CommandResult<Snapshot> {
    let Snapshot {
        snapshot_id,
        timestamp,
        active_bundle_id,
        snapshot_generation,
        elements: snapshot_elements,
    } = snapshot;

    let elements = if let Some(scope_target) = scope {
        extract_subtree(&snapshot_elements, scope_target)?
    } else {
        snapshot_elements
    };

    let elements = filter_snapshot_elements(&elements, interactive, compact, depth);

    Ok(Snapshot {
        snapshot_id,
        timestamp,
        active_bundle_id,
        snapshot_generation,
        elements,
    })
}

fn filter_snapshot_elements(
    elements: &[types::SnapshotElement],
    interactive: bool,
    compact: bool,
    depth: Option<u32>,
) -> Vec<types::SnapshotElement> {
    let included_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(index, _)| should_keep_element(elements, *index, interactive, compact, depth))
        .map(|(index, _)| index)
        .collect();

    remap_elements(elements, &included_indices, 0)
}

fn should_keep_element(
    elements: &[types::SnapshotElement],
    index: usize,
    interactive: bool,
    compact: bool,
    depth: Option<u32>,
) -> bool {
    let element = &elements[index];

    if let Some(max_depth) = depth {
        if element.depth > max_depth {
            return false;
        }
    }

    if interactive
        && !element.is_interactive
        && !ref_generator::has_interactive_descendants(elements, index)
    {
        return false;
    }

    if compact && element.is_empty_structure() && element.children_indices.is_empty() {
        return false;
    }

    true
}

fn remap_elements(
    elements: &[types::SnapshotElement],
    included_indices: &[usize],
    depth_offset: u32,
) -> Vec<types::SnapshotElement> {
    use std::collections::HashMap;

    let index_map: HashMap<usize, usize> = included_indices
        .iter()
        .enumerate()
        .map(|(new_index, &old_index)| (old_index, new_index))
        .collect();

    included_indices
        .iter()
        .map(|&old_index| {
            let mut element = elements[old_index].clone();
            element.depth = element.depth.saturating_sub(depth_offset);
            element.parent_index = element
                .parent_index
                .and_then(|parent| index_map.get(&parent).copied());
            element.children_indices = element
                .children_indices
                .iter()
                .filter_map(|child| index_map.get(child).copied())
                .collect();
            element
        })
        .collect()
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

    Ok(remap_elements(elements, &included_indices, root_depth))
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
    use agent_mobile_core::snapshot::Frame;
    use agent_mobile_platform_ios::xcuitest::types::{
        RunnerSnapshotElement, RunnerSnapshotResponse,
    };

    #[test]
    fn test_generate_snapshot_id() {
        let id1 = generate_snapshot_id();
        let id2 = generate_snapshot_id();

        assert!(id1.starts_with("snap_"));
        assert!(id2.starts_with("snap_"));
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 13); // "snap_" + 8 chars
    }

    fn sample_element(
        ref_id: &str,
        element_type: &str,
        depth: u32,
        parent_index: Option<usize>,
        children_indices: Vec<usize>,
        is_interactive: bool,
        label: Option<&str>,
    ) -> types::SnapshotElement {
        types::SnapshotElement {
            ref_id: ref_id.to_string(),
            element_id: Some(format!("{}-{}", element_type, ref_id)),
            element_type: element_type.to_string(),
            label: label.map(str::to_string),
            frame: Frame {
                x: depth as f64 * 10.0,
                y: depth as f64 * 10.0,
                width: 100.0,
                height: 40.0,
            },
            enabled: true,
            traits: extract_traits_for_type(element_type),
            placeholder: None,
            value: None,
            children_indices,
            depth,
            is_interactive,
            parent_index,
        }
    }

    fn sample_snapshot() -> Snapshot {
        Snapshot {
            snapshot_id: "snap_test".to_string(),
            timestamp: Utc::now(),
            active_bundle_id: Some("com.apple.Preferences".to_string()),
            snapshot_generation: Some(9),
            elements: vec![
                sample_element("@e1", "Application", 0, None, vec![1, 2], false, None),
                sample_element(
                    "@e2",
                    "StaticText",
                    1,
                    Some(0),
                    vec![],
                    false,
                    Some("General"),
                ),
                sample_element("@e3", "Button", 1, Some(0), vec![], true, Some("About")),
            ],
        }
    }

    #[test]
    fn test_finalize_snapshot_applies_depth_interactive_and_compact() {
        let snapshot = sample_snapshot();
        let filtered = finalize_snapshot(snapshot, true, true, Some(1), None).unwrap();

        assert_eq!(filtered.elements.len(), 2);
        assert_eq!(filtered.elements[0].ref_id, "@e1");
        assert_eq!(filtered.elements[1].ref_id, "@e3");
        assert_eq!(filtered.elements[0].depth, 0);
        assert_eq!(filtered.elements[1].depth, 1);
        assert_eq!(filtered.elements[0].children_indices, vec![1]);
        assert_eq!(filtered.elements[1].parent_index, Some(0));
    }

    #[test]
    fn test_snapshot_from_runner_response_preserves_metadata_and_hierarchy() {
        let response = RunnerSnapshotResponse {
            snapshot_id: "snap_runner".to_string(),
            active_bundle_id: Some("com.apple.Preferences".to_string()),
            snapshot_generation: 4,
            elements: vec![
                RunnerSnapshotElement {
                    element_id: "app".to_string(),
                    element_type: "Application".to_string(),
                    label: None,
                    value: None,
                    placeholder: None,
                    frame: Frame {
                        x: 0.0,
                        y: 0.0,
                        width: 393.0,
                        height: 852.0,
                    },
                    enabled: true,
                    interactive: false,
                    depth: Some(0),
                },
                RunnerSnapshotElement {
                    element_id: "general-cell".to_string(),
                    element_type: "Cell".to_string(),
                    label: Some("General".to_string()),
                    value: None,
                    placeholder: None,
                    frame: Frame {
                        x: 0.0,
                        y: 100.0,
                        width: 393.0,
                        height: 44.0,
                    },
                    enabled: true,
                    interactive: true,
                    depth: Some(1),
                },
            ],
        };

        let snapshot = snapshot_from_runner_response(response);
        assert_eq!(snapshot.snapshot_id, "snap_runner");
        assert_eq!(
            snapshot.active_bundle_id.as_deref(),
            Some("com.apple.Preferences")
        );
        assert_eq!(snapshot.snapshot_generation, Some(4));
        assert_eq!(snapshot.elements.len(), 2);
        assert_eq!(snapshot.elements[0].children_indices, vec![1]);
        assert_eq!(snapshot.elements[1].parent_index, Some(0));
        assert_eq!(
            snapshot.elements[1].element_id.as_deref(),
            Some("general-cell")
        );
        assert_eq!(snapshot.elements[0].ref_id, "@e2");
        assert_eq!(snapshot.elements[1].ref_id, "@e1");
    }
}
