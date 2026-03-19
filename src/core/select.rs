//! select command - Select value from Picker/Spinner
//!
//! ```bash
//! agent-mobile select @e1 "Option 2"       # Select value by Picker/@eN
//! agent-mobile select "Country" "Japan"    # Select value by Picker/text
//! agent-mobile select @e1 "Option 2" --max-swipes 15
//! ```

use std::time::Duration;

use clap::Args;
use serde::Serialize;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::{prepare_xcuitest_with_policy, AppContextPolicy, CommandResult};
use crate::helpers::common_args::DeviceArgs;
use crate::helpers::format::OutputFormat;
use crate::helpers::ios::get_ios_screen_size_from_client;
use crate::snapshot::types::{Snapshot, SnapshotElement};

use super::ref_resolver::ElementTarget;
use super::swipe::execute_ios_swipe;
use super::tap::{
    execute_ios_tap, execute_tap, resolve_element, resolve_ios_element_with_client,
    take_ios_snapshot_with_client, take_snapshot,
};

/// Arguments for the select command
#[derive(Args, Debug)]
pub struct SelectArgs {
    /// Target picker/spinner: @eN ref or "text"
    pub target: String,

    /// Value to select
    pub value: String,

    /// Maximum number of swipe attempts to find the value
    #[arg(long, default_value = "10")]
    pub max_swipes: u32,

    /// Output format (text or json)
    #[arg(short = 'f', long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Device selection options.
    #[command(flatten)]
    pub device: DeviceArgs,
}

/// JSON output for select command
#[derive(Debug, Serialize)]
struct SelectOutput {
    target: String,
    value: String,
    status: String,
}

/// Execute the select command
pub async fn run(args: SelectArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => run_android(args).await,
    }
}

/// Execute select on iOS (Picker wheel)
async fn run_ios(args: SelectArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);
    let value_lower = args.value.to_lowercase();
    let format = args.format;
    let (resolved_udid, client, _) = prepare_xcuitest_with_policy(
        args.device.udid.as_deref(),
        AppContextPolicy::RestoreIfUnset,
    )
    .await?;

    // 1. Find and tap the picker to activate it
    let picker_element = resolve_ios_element_with_client(&target, &resolved_udid, &client).await?;
    let (picker_x, picker_y) = picker_element.center();

    execute_ios_tap(&client, picker_x, picker_y).await?;

    // Wait for picker to open
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Try to find and select the value by scrolling the picker
    for attempt in 0..args.max_swipes {
        // Take a fresh snapshot
        let snapshot = take_ios_snapshot_with_client(&client).await?;
        let picker_region = picker_region(&snapshot, &picker_element.frame);

        // Look for the value in the current snapshot
        if let Some(element) = find_picker_value(&snapshot, &picker_region, &value_lower) {
            // Found it - tap to select
            let (x, y) = element.frame.center();
            execute_ios_tap(&client, x, y).await?;

            // Wait a moment then try to dismiss (tap "Done" or outside)
            tokio::time::sleep(Duration::from_millis(300)).await;

            let post_select_snapshot = take_ios_snapshot_with_client(&client).await?;
            dismiss_picker(&client, &post_select_snapshot, &picker_region).await?;

            if !picker_value_updated(&target, &resolved_udid, &client, &value_lower).await? {
                return Err(format!("Picker value did not update to '{}'", args.value).into());
            }

            if format.is_json() {
                let output = SelectOutput {
                    target: args.target.clone(),
                    value: args.value.clone(),
                    status: "success".to_string(),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Selected: {}", args.value);
            }
            return Ok(());
        }

        // Not found yet - swipe the picker wheel
        // Alternate between up and down to cover both directions
        let swipe_direction = if attempt % 2 == 0 { -50.0 } else { 50.0 };
        execute_ios_swipe(
            &client,
            picker_x,
            picker_y,
            picker_x,
            picker_y + swipe_direction,
            0.2,
        )
        .await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    Err(format!(
        "Could not find '{}' in picker after {} swipes",
        args.value, args.max_swipes
    )
    .into())
}

/// Execute select on Android (Spinner dropdown)
async fn run_android(args: SelectArgs) -> CommandResult {
    let target = ElementTarget::parse(&args.target);
    let value_lower = args.value.to_lowercase();
    let udid = args.device.udid.as_deref();
    let format = args.format;

    // 1. Find and tap the spinner to open dropdown
    let spinner_element = resolve_element(&target, Platform::Android, udid).await?;
    let (spinner_x, spinner_y) = spinner_element.center();

    execute_tap(Platform::Android, udid, spinner_x, spinner_y).await?;

    // Wait for dropdown to open
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Find and tap the value in the dropdown
    for attempt in 0..args.max_swipes {
        let snapshot = take_snapshot(Platform::Android, udid).await?;

        // Look for the value in the dropdown list
        if let Some(element) = snapshot.elements.iter().find(|e| {
            e.label
                .as_ref()
                .map(|l| l.to_lowercase().contains(&value_lower))
                .unwrap_or(false)
                || e.value
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&value_lower))
                    .unwrap_or(false)
        }) {
            // Found it - tap to select
            let (x, y) = element.frame.center();
            execute_tap(Platform::Android, udid, x, y).await?;

            if format.is_json() {
                let output = SelectOutput {
                    target: args.target.clone(),
                    value: args.value.clone(),
                    status: "success".to_string(),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Selected: {}", args.value);
            }
            return Ok(());
        }

        // Not found - scroll down in the dropdown
        if attempt < args.max_swipes - 1 {
            // Get screen dimensions for scroll
            let (screen_width, screen_height) =
                agent_mobile_platform_android::adb::input::get_screen_size(udid)
                    .await
                    .unwrap_or((1080, 1920));

            let center_x = screen_width as f64 / 2.0;
            let start_y = screen_height as f64 * 0.6;
            let end_y = screen_height as f64 * 0.4;

            execute_swipe_android(udid, center_x, start_y, center_x, end_y, Some(0.3)).await?;

            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }

    Err(format!(
        "Could not find '{}' in spinner dropdown after {} scroll attempts",
        args.value, args.max_swipes
    )
    .into())
}

/// Execute swipe on Android
async fn execute_swipe_android(
    udid: Option<&str>,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    duration: Option<f64>,
) -> CommandResult {
    use agent_mobile_platform_android::adb::input;

    let duration_ms = duration.map(|d| (d * 1000.0) as u64);
    input::swipe(udid, x1, y1, x2, y2, duration_ms).await?;
    Ok(())
}

fn find_picker_value<'a>(
    snapshot: &'a Snapshot,
    picker_region: &agent_mobile_core::snapshot::Frame,
    value_lower: &str,
) -> Option<&'a SnapshotElement> {
    snapshot
        .elements
        .iter()
        .filter(|element| frame_intersects(&element.frame, picker_region))
        .filter(|element| element_matches_value(element, value_lower))
        .min_by(|left, right| {
            let left_delta = distance_to_region_center(&left.frame, picker_region);
            let right_delta = distance_to_region_center(&right.frame, picker_region);
            left_delta.total_cmp(&right_delta)
        })
}

fn picker_region(
    snapshot: &Snapshot,
    fallback: &agent_mobile_core::snapshot::Frame,
) -> agent_mobile_core::snapshot::Frame {
    snapshot
        .elements
        .iter()
        .filter(|element| {
            element.frame.width >= fallback.width.max(120.0)
                && element.frame.height >= fallback.height.max(120.0)
                && frame_contains_point(&element.frame, fallback.center())
        })
        .min_by(|left, right| {
            let left_area = left.frame.width * left.frame.height;
            let right_area = right.frame.width * right.frame.height;
            left_area.total_cmp(&right_area)
        })
        .map(|element| element.frame.clone())
        .unwrap_or_else(|| fallback.clone())
}

async fn dismiss_picker(
    client: &agent_mobile_platform_ios::xcuitest::XCUITestClient,
    snapshot: &Snapshot,
    picker_region: &agent_mobile_core::snapshot::Frame,
) -> CommandResult {
    if let Some(done_btn) = find_picker_done_button(snapshot, picker_region) {
        let (x, y) = done_btn.frame.center();
        execute_ios_tap(client, x, y).await?;
        return Ok(());
    }

    let (screen_width, _) = get_ios_screen_size_from_client(client).await?;
    let tap_x = (picker_region.x + picker_region.width / 2.0).clamp(1.0, screen_width - 1.0);
    let tap_y = if picker_region.y > 44.0 {
        (picker_region.y - 24.0).max(1.0)
    } else {
        (picker_region.y + picker_region.height + 24.0).max(1.0)
    };
    execute_ios_tap(client, tap_x, tap_y).await
}

fn find_picker_done_button<'a>(
    snapshot: &'a Snapshot,
    picker_region: &agent_mobile_core::snapshot::Frame,
) -> Option<&'a SnapshotElement> {
    let right_edge = picker_region.x + picker_region.width;
    let top_limit = picker_region.y + (picker_region.height * 0.35);

    snapshot
        .elements
        .iter()
        .filter(|element| {
            (element.element_type == "Button"
                || element
                    .traits
                    .iter()
                    .any(|trait_name| trait_name == "button"))
                && frame_intersects(&element.frame, picker_region)
                && element.frame.y <= top_limit
                && element.frame.x + element.frame.width / 2.0
                    >= right_edge - picker_region.width * 0.35
        })
        .min_by(|left, right| left.frame.y.total_cmp(&right.frame.y))
        .or_else(|| {
            snapshot.elements.iter().find(|element| {
                element
                    .label
                    .as_ref()
                    .map(|label| label == "Done" || label == "完了")
                    .unwrap_or(false)
            })
        })
}

async fn picker_value_updated(
    target: &ElementTarget,
    resolved_udid: &str,
    client: &agent_mobile_platform_ios::xcuitest::XCUITestClient,
    value_lower: &str,
) -> CommandResult<bool> {
    let updated = resolve_ios_element_with_client(target, resolved_udid, client).await?;
    Ok(updated
        .value
        .as_ref()
        .or(updated.label.as_ref())
        .map(|value| value.to_lowercase().contains(value_lower))
        .unwrap_or(false))
}

fn element_matches_value(element: &SnapshotElement, value_lower: &str) -> bool {
    element
        .label
        .as_ref()
        .map(|label| label.to_lowercase().contains(value_lower))
        .unwrap_or(false)
        || element
            .value
            .as_ref()
            .map(|value| value.to_lowercase().contains(value_lower))
            .unwrap_or(false)
}

fn distance_to_region_center(
    frame: &agent_mobile_core::snapshot::Frame,
    region: &agent_mobile_core::snapshot::Frame,
) -> f64 {
    let (frame_x, frame_y) = frame.center();
    let (region_x, region_y) = region.center();
    (frame_x - region_x).abs() + (frame_y - region_y).abs()
}

fn frame_contains_point(frame: &agent_mobile_core::snapshot::Frame, point: (f64, f64)) -> bool {
    point.0 >= frame.x
        && point.0 <= frame.x + frame.width
        && point.1 >= frame.y
        && point.1 <= frame.y + frame.height
}

fn frame_intersects(
    left: &agent_mobile_core::snapshot::Frame,
    right: &agent_mobile_core::snapshot::Frame,
) -> bool {
    left.x < right.x + right.width
        && left.x + left.width > right.x
        && left.y < right.y + right.height
        && left.y + left.height > right.y
}

#[cfg(test)]
mod tests {
    use agent_mobile_core::snapshot::Frame;
    use chrono::Utc;

    use super::*;

    fn snapshot_with_elements(elements: Vec<SnapshotElement>) -> Snapshot {
        Snapshot {
            snapshot_id: "snap_test".to_string(),
            timestamp: Utc::now(),
            active_bundle_id: None,
            snapshot_generation: None,
            elements,
        }
    }

    fn test_element(
        ref_id: &str,
        label: Option<&str>,
        frame: Frame,
        element_type: &str,
    ) -> SnapshotElement {
        SnapshotElement {
            ref_id: ref_id.to_string(),
            element_id: None,
            element_type: element_type.to_string(),
            label: label.map(str::to_string),
            frame,
            enabled: true,
            traits: if element_type == "Button" {
                vec!["button".to_string()]
            } else {
                Vec::new()
            },
            placeholder: None,
            value: None,
            children_indices: Vec::new(),
            depth: 0,
            is_interactive: true,
            parent_index: None,
        }
    }

    #[test]
    fn finds_picker_value_inside_picker_region() {
        let picker = Frame {
            x: 20.0,
            y: 400.0,
            width: 350.0,
            height: 220.0,
        };
        let snapshot = snapshot_with_elements(vec![
            test_element(
                "@e1",
                Some("Japan"),
                Frame {
                    x: 40.0,
                    y: 470.0,
                    width: 300.0,
                    height: 40.0,
                },
                "StaticText",
            ),
            test_element(
                "@e2",
                Some("Japan"),
                Frame {
                    x: 40.0,
                    y: 80.0,
                    width: 100.0,
                    height: 40.0,
                },
                "StaticText",
            ),
        ]);

        let found = find_picker_value(&snapshot, &picker, "japan").unwrap();
        assert_eq!(found.ref_id, "@e1");
    }

    #[test]
    fn prefers_top_right_button_for_picker_done() {
        let picker = Frame {
            x: 20.0,
            y: 400.0,
            width: 350.0,
            height: 220.0,
        };
        let snapshot = snapshot_with_elements(vec![
            test_element(
                "@cancel",
                Some("Cancel"),
                Frame {
                    x: 40.0,
                    y: 420.0,
                    width: 80.0,
                    height: 40.0,
                },
                "Button",
            ),
            test_element(
                "@done",
                Some("完了"),
                Frame {
                    x: 280.0,
                    y: 420.0,
                    width: 70.0,
                    height: 40.0,
                },
                "Button",
            ),
        ]);

        let found = find_picker_done_button(&snapshot, &picker).unwrap();
        assert_eq!(found.ref_id, "@done");
    }
}
