//! find コマンド - semantic locators を使った UI 要素検索とアクション実行
//!
//! ```bash
//! # 要素検索のみ (ref を返す)
//! agent-mobile find type Button
//! agent-mobile find text "Login"
//! agent-mobile find label "Email" --first
//!
//! # 検索 + アクション実行
//! agent-mobile find type Button tap
//! agent-mobile find text "Login" tap
//! agent-mobile find label "Email" fill "test@example.com"
//! agent-mobile find placeholder "Search..." fill "query"
//! agent-mobile find text "Submit" long-press
//!
//! # 位置指定 + アクション
//! agent-mobile find type Button --nth 2 tap
//! agent-mobile find text "Option" --last tap
//!
//! # JSON 出力 (アクションなし時)
//! agent-mobile find type Button -f json --all
//! ```

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use agent_mobile_core::snapshot::Frame;
use agent_mobile_core::Platform;

use crate::helpers::{with_client, CommandResult, DeviceArgs};
use crate::snapshot::types::{Snapshot, SnapshotElement};

use super::tap::take_snapshot;
use agent_mobile_gateway::DeviceResolver;

/// find コマンド引数
#[derive(Args, Debug)]
pub struct FindArgs {
    #[command(subcommand)]
    pub locator: FindLocator,

    /// Select only the first matching element (default)
    #[arg(long, global = true, conflicts_with = "last", conflicts_with = "nth")]
    pub first: bool,

    /// Select only the last matching element
    #[arg(long, global = true, conflicts_with = "first", conflicts_with = "nth")]
    pub last: bool,

    /// Select the Nth matching element (0-indexed)
    #[arg(long, global = true, conflicts_with = "first", conflicts_with = "last")]
    pub nth: Option<usize>,

    /// Return all matching elements (only when no action specified)
    #[arg(short = 'a', long, global = true)]
    pub all: bool,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "text", global = true)]
    pub format: FindOutputFormat,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Locator types for element search
#[derive(Subcommand, Debug)]
pub enum FindLocator {
    /// Find by element type (Button, TextField, etc.)
    Type {
        /// Element type to search for (case-insensitive)
        element_type: String,

        /// Action to perform on found element
        action: Option<String>,

        /// Value for fill action
        action_value: Option<String>,
    },

    /// Find by text content (label or value)
    Text {
        /// Text to search for
        text: String,

        /// Require exact match (default: partial match)
        #[arg(long)]
        exact: bool,

        /// Action to perform on found element
        action: Option<String>,

        /// Value for fill action
        action_value: Option<String>,
    },

    /// Find by label attribute
    Label {
        /// Label text to search for
        label: String,

        /// Require exact match (default: partial match)
        #[arg(long)]
        exact: bool,

        /// Action to perform on found element
        action: Option<String>,

        /// Value for fill action
        action_value: Option<String>,
    },

    /// Find by placeholder text
    Placeholder {
        /// Placeholder text to search for
        placeholder: String,

        /// Require exact match (default: partial match)
        #[arg(long)]
        exact: bool,

        /// Action to perform on found element
        action: Option<String>,

        /// Value for fill action
        action_value: Option<String>,
    },

    /// Find enabled elements
    Enabled {
        /// Action to perform on found element
        action: Option<String>,

        /// Value for fill action
        action_value: Option<String>,
    },

    /// Find disabled elements
    Disabled {
        /// Action to perform on found element
        action: Option<String>,

        /// Value for fill action
        action_value: Option<String>,
    },
}

/// Output format for find command
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FindOutputFormat {
    /// Plain text output (just ref)
    Text,
    /// JSON formatted output
    Json,
}

/// Parsed action from command arguments
#[derive(Debug, Clone)]
pub enum FindAction {
    /// Tap the element
    Tap,
    /// Long press the element
    LongPress,
    /// Fill text field (clear + type)
    Fill(String),
    /// Clear text field
    Clear,
}

impl FindAction {
    /// Parse action from string
    fn parse(action: &str, action_value: Option<&str>) -> Result<Self, String> {
        match action.to_lowercase().as_str() {
            "tap" => Ok(FindAction::Tap),
            "long-press" | "longpress" => Ok(FindAction::LongPress),
            "fill" => {
                let value =
                    action_value.ok_or_else(|| "fill action requires a value".to_string())?;
                Ok(FindAction::Fill(value.to_string()))
            }
            "clear" => Ok(FindAction::Clear),
            _ => Err(format!(
                "Unknown action: '{}'. Valid actions: tap, long-press, fill, clear",
                action
            )),
        }
    }
}

/// JSON output for a single element
#[derive(Debug, Serialize)]
struct ElementOutput {
    #[serde(rename = "ref")]
    ref_id: String,
    #[serde(rename = "type")]
    element_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    enabled: bool,
    frame: Frame,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
}

impl From<&SnapshotElement> for ElementOutput {
    fn from(elem: &SnapshotElement) -> Self {
        Self {
            ref_id: elem.ref_id.clone(),
            element_type: elem.element_type.clone(),
            label: elem.label.clone(),
            enabled: elem.enabled,
            frame: elem.frame.clone(),
            value: elem.value.clone(),
            placeholder: elem.placeholder.clone(),
        }
    }
}

/// Execute the find command
pub async fn run(args: FindArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref()).await?;

    // Extract action from locator
    let (action_str, action_value) = extract_action(&args.locator);
    let action = if let Some(action_str) = action_str {
        Some(FindAction::parse(action_str, action_value.as_deref())?)
    } else {
        None
    };

    // Validate: --all cannot be used with an action
    if args.all && action.is_some() {
        return Err("--all cannot be used with an action (tap, fill, etc.)".into());
    }

    // Get snapshot
    let snapshot = take_snapshot(platform, args.device.udid.as_deref()).await?;

    // Find matching elements
    let matches = find_elements(&snapshot, &args.locator);

    if matches.is_empty() {
        return Err(format!(
            "No elements found matching {}\n\nHint: Run 'agent-mobile snapshot' to see the current UI state.",
            describe_locator(&args.locator)
        ).into());
    }

    // Select element(s) based on position flags
    let selected = select_elements(&matches, args.first, args.last, args.nth, args.all)?;

    // If action is specified, execute it
    if let Some(action) = action {
        if selected.len() > 1 {
            return Err("Cannot perform action on multiple elements. Use --first, --last, or --nth to select one.".into());
        }
        let element = selected[0];
        execute_action(platform, args.device.udid.as_deref(), element, &action).await?;

        // Output action result
        let action_desc = match &action {
            FindAction::Tap => "Tapped".to_string(),
            FindAction::LongPress => "Long-pressed".to_string(),
            FindAction::Fill(text) => format!("Filled with \"{}\"", text),
            FindAction::Clear => "Cleared".to_string(),
        };
        let element_desc = format_element_short(element);
        println!("Found {}", element_desc);
        println!("{} {}", action_desc, element.ref_id);
    } else {
        // Output found element(s)
        output_elements(&selected, args.format)?;
    }

    Ok(())
}

/// Extract action and action_value from locator
fn extract_action(locator: &FindLocator) -> (Option<&str>, Option<String>) {
    match locator {
        FindLocator::Type {
            action,
            action_value,
            ..
        }
        | FindLocator::Text {
            action,
            action_value,
            ..
        }
        | FindLocator::Label {
            action,
            action_value,
            ..
        }
        | FindLocator::Placeholder {
            action,
            action_value,
            ..
        }
        | FindLocator::Enabled {
            action,
            action_value,
        }
        | FindLocator::Disabled {
            action,
            action_value,
        } => (action.as_deref(), action_value.clone()),
    }
}

/// Describe the locator for error messages
fn describe_locator(locator: &FindLocator) -> String {
    match locator {
        FindLocator::Type { element_type, .. } => format!("type '{}'", element_type),
        FindLocator::Text { text, exact, .. } => {
            if *exact {
                format!("text exactly \"{}\"", text)
            } else {
                format!("text containing \"{}\"", text)
            }
        }
        FindLocator::Label { label, exact, .. } => {
            if *exact {
                format!("label exactly \"{}\"", label)
            } else {
                format!("label containing \"{}\"", label)
            }
        }
        FindLocator::Placeholder {
            placeholder, exact, ..
        } => {
            if *exact {
                format!("placeholder exactly \"{}\"", placeholder)
            } else {
                format!("placeholder containing \"{}\"", placeholder)
            }
        }
        FindLocator::Enabled { .. } => "enabled elements".to_string(),
        FindLocator::Disabled { .. } => "disabled elements".to_string(),
    }
}

/// Find elements matching the locator
fn find_elements<'a>(snapshot: &'a Snapshot, locator: &FindLocator) -> Vec<&'a SnapshotElement> {
    snapshot
        .elements
        .iter()
        .filter(|elem| matches_locator(elem, locator))
        .collect()
}

/// Check if an element matches the locator
fn matches_locator(elem: &SnapshotElement, locator: &FindLocator) -> bool {
    match locator {
        FindLocator::Type { element_type, .. } => {
            elem.element_type.to_lowercase() == element_type.to_lowercase()
        }
        FindLocator::Text { text, exact, .. } => {
            let text_lower = text.to_lowercase();
            if *exact {
                elem.label
                    .as_ref()
                    .map(|l| l.to_lowercase() == text_lower)
                    .unwrap_or(false)
                    || elem
                        .value
                        .as_ref()
                        .map(|v| v.to_lowercase() == text_lower)
                        .unwrap_or(false)
            } else {
                elem.label
                    .as_ref()
                    .map(|l| l.to_lowercase().contains(&text_lower))
                    .unwrap_or(false)
                    || elem
                        .value
                        .as_ref()
                        .map(|v| v.to_lowercase().contains(&text_lower))
                        .unwrap_or(false)
            }
        }
        FindLocator::Label { label, exact, .. } => {
            let label_lower = label.to_lowercase();
            if *exact {
                elem.label
                    .as_ref()
                    .map(|l| l.to_lowercase() == label_lower)
                    .unwrap_or(false)
            } else {
                elem.label
                    .as_ref()
                    .map(|l| l.to_lowercase().contains(&label_lower))
                    .unwrap_or(false)
            }
        }
        FindLocator::Placeholder {
            placeholder, exact, ..
        } => {
            let placeholder_lower = placeholder.to_lowercase();
            if *exact {
                elem.placeholder
                    .as_ref()
                    .map(|p| p.to_lowercase() == placeholder_lower)
                    .unwrap_or(false)
            } else {
                elem.placeholder
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&placeholder_lower))
                    .unwrap_or(false)
            }
        }
        FindLocator::Enabled { .. } => elem.enabled,
        FindLocator::Disabled { .. } => !elem.enabled,
    }
}

/// Select elements based on position flags
fn select_elements<'a>(
    elements: &[&'a SnapshotElement],
    _first: bool,
    last: bool,
    nth: Option<usize>,
    all: bool,
) -> CommandResult<Vec<&'a SnapshotElement>> {
    if all {
        return Ok(elements.to_vec());
    }

    if let Some(n) = nth {
        if n >= elements.len() {
            return Err(format!(
                "Index {} out of range. Found {} element(s).",
                n,
                elements.len()
            )
            .into());
        }
        return Ok(vec![elements[n]]);
    }

    if last {
        return Ok(vec![elements[elements.len() - 1]]);
    }

    // Default to first (when --first is specified or no flag is given)
    Ok(vec![elements[0]])
}

/// Format element for short display
fn format_element_short(elem: &SnapshotElement) -> String {
    let label_part = elem
        .label
        .as_ref()
        .map(|l| format!(" \"{}\"", l))
        .unwrap_or_default();
    format!("{} ({}{})", elem.ref_id, elem.element_type, label_part)
}

/// Output elements in the specified format
fn output_elements(elements: &[&SnapshotElement], format: FindOutputFormat) -> CommandResult {
    match format {
        FindOutputFormat::Text => {
            for elem in elements {
                println!("{}", elem.ref_id);
            }
        }
        FindOutputFormat::Json => {
            if elements.len() == 1 {
                let output = ElementOutput::from(elements[0]);
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                let outputs: Vec<ElementOutput> =
                    elements.iter().map(|e| ElementOutput::from(*e)).collect();
                println!("{}", serde_json::to_string_pretty(&outputs)?);
            }
        }
    }
    Ok(())
}

/// Execute action on the element
async fn execute_action(
    platform: Platform,
    udid: Option<&str>,
    element: &SnapshotElement,
    action: &FindAction,
) -> CommandResult {
    let (x, y) = element.frame.center();

    match action {
        FindAction::Tap => execute_tap(platform, udid, x, y).await,
        FindAction::LongPress => execute_long_press(platform, udid, x, y, 1.0).await,
        FindAction::Fill(text) => {
            let clear_len = element
                .value
                .as_ref()
                .map(|v| v.chars().count())
                .unwrap_or(50);
            execute_fill(platform, udid, x, y, text, clear_len).await
        }
        FindAction::Clear => {
            let clear_len = element
                .value
                .as_ref()
                .map(|v| v.chars().count())
                .unwrap_or(50);
            execute_clear(platform, udid, x, y, clear_len).await
        }
    }
}

/// Execute tap gesture
async fn execute_tap(platform: Platform, udid: Option<&str>, x: f64, y: f64) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::idb::hid::events;

            with_client(udid, |mut client| async move {
                let events = events::tap_to_events(x, y, None);
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::input;
            input::tap(udid, x, y).await?;
            Ok(())
        }
    }
}

/// Execute long press gesture
async fn execute_long_press(
    platform: Platform,
    udid: Option<&str>,
    x: f64,
    y: f64,
    duration: f64,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::idb::hid::events;

            with_client(udid, |mut client| async move {
                let events = events::tap_to_events(x, y, Some(duration));
                client.hid(events).await?;
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::input;
            let duration_ms = (duration * 1000.0) as u64;
            input::long_press(udid, x, y, duration_ms).await?;
            Ok(())
        }
    }
}

/// Execute fill (tap + clear + type)
async fn execute_fill(
    platform: Platform,
    udid: Option<&str>,
    x: f64,
    y: f64,
    text: &str,
    clear_len: usize,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::idb::hid::events;
            let text = text.to_string();

            with_client(udid, |mut client| async move {
                // 1. Tap to focus
                let tap_events = events::tap_to_events(x, y, None);
                client.hid(tap_events).await?;

                // Small delay to ensure focus
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                // 2. Clear existing text
                for _ in 0..clear_len {
                    let del_events = events::key_to_events(42, None); // BACKSPACE = 42
                    client.hid(del_events).await?;
                }

                // 3. Type new text
                let text_events = events::text_to_events(&text)?;
                client.hid(text_events).await?;

                Ok(())
            })
            .await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::input;

            // 1. Tap to focus
            input::tap(udid, x, y).await?;

            // Small delay to ensure focus
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // 2. Clear existing text
            input::keyevent_by_name(udid, "KEYCODE_MOVE_END").await?;
            for _ in 0..clear_len {
                input::keyevent(udid, input::keycodes::DEL).await?;
            }

            // 3. Type new text
            input::text(udid, text).await?;

            Ok(())
        }
    }
}

/// Execute clear (tap + select all + delete)
async fn execute_clear(
    platform: Platform,
    udid: Option<&str>,
    x: f64,
    y: f64,
    clear_len: usize,
) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::idb::hid::events;

            with_client(udid, |mut client| async move {
                // 1. Tap to focus
                let tap_events = events::tap_to_events(x, y, None);
                client.hid(tap_events).await?;

                // Small delay to ensure focus
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                // 2. Clear existing text
                for _ in 0..clear_len {
                    let del_events = events::key_to_events(42, None); // BACKSPACE = 42
                    client.hid(del_events).await?;
                }

                Ok(())
            })
            .await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::input;

            // 1. Tap to focus
            input::tap(udid, x, y).await?;

            // Small delay to ensure focus
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // 2. Clear existing text
            input::keyevent_by_name(udid, "KEYCODE_MOVE_END").await?;
            for _ in 0..clear_len {
                input::keyevent(udid, input::keycodes::DEL).await?;
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::types::Frame;
    use chrono::Utc;

    fn create_test_snapshot() -> Snapshot {
        Snapshot {
            snapshot_id: "test_snap".to_string(),
            timestamp: Utc::now(),
            elements: vec![
                SnapshotElement {
                    ref_id: "@e1".to_string(),
                    element_type: "Button".to_string(),
                    label: Some("Login".to_string()),
                    frame: Frame {
                        x: 100.0,
                        y: 200.0,
                        width: 80.0,
                        height: 44.0,
                    },
                    enabled: true,
                    traits: vec![],
                    placeholder: None,
                    value: None,
                    children_indices: vec![],
                    depth: 0,
                    is_interactive: true,
                    parent_index: None,
                },
                SnapshotElement {
                    ref_id: "@e2".to_string(),
                    element_type: "TextField".to_string(),
                    label: Some("Email".to_string()),
                    frame: Frame {
                        x: 50.0,
                        y: 100.0,
                        width: 200.0,
                        height: 40.0,
                    },
                    enabled: true,
                    traits: vec![],
                    placeholder: Some("Enter your email".to_string()),
                    value: Some("test@example.com".to_string()),
                    children_indices: vec![],
                    depth: 0,
                    is_interactive: true,
                    parent_index: None,
                },
                SnapshotElement {
                    ref_id: "@e3".to_string(),
                    element_type: "Button".to_string(),
                    label: Some("Submit".to_string()),
                    frame: Frame {
                        x: 100.0,
                        y: 300.0,
                        width: 80.0,
                        height: 44.0,
                    },
                    enabled: false,
                    traits: vec![],
                    placeholder: None,
                    value: None,
                    children_indices: vec![],
                    depth: 0,
                    is_interactive: true,
                    parent_index: None,
                },
                SnapshotElement {
                    ref_id: "@e4".to_string(),
                    element_type: "Button".to_string(),
                    label: Some("Cancel".to_string()),
                    frame: Frame {
                        x: 200.0,
                        y: 300.0,
                        width: 80.0,
                        height: 44.0,
                    },
                    enabled: true,
                    traits: vec![],
                    placeholder: None,
                    value: None,
                    children_indices: vec![],
                    depth: 0,
                    is_interactive: true,
                    parent_index: None,
                },
            ],
        }
    }

    #[test]
    fn test_find_by_type() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Type {
            element_type: "Button".to_string(),
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].ref_id, "@e1");
        assert_eq!(matches[1].ref_id, "@e3");
        assert_eq!(matches[2].ref_id, "@e4");
    }

    #[test]
    fn test_find_by_type_case_insensitive() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Type {
            element_type: "button".to_string(),
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_find_by_text_contains() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Text {
            text: "Log".to_string(),
            exact: false,
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ref_id, "@e1");
    }

    #[test]
    fn test_find_by_text_exact() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Text {
            text: "Log".to_string(),
            exact: true,
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 0);

        let locator = FindLocator::Text {
            text: "Login".to_string(),
            exact: true,
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ref_id, "@e1");
    }

    #[test]
    fn test_find_by_label() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Label {
            label: "Email".to_string(),
            exact: false,
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ref_id, "@e2");
    }

    #[test]
    fn test_find_by_placeholder() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Placeholder {
            placeholder: "email".to_string(),
            exact: false,
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ref_id, "@e2");
    }

    #[test]
    fn test_find_enabled() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Enabled {
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_find_disabled() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Disabled {
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ref_id, "@e3");
    }

    #[test]
    fn test_select_first() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Type {
            element_type: "Button".to_string(),
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        let selected = select_elements(&matches, true, false, None, false).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].ref_id, "@e1");
    }

    #[test]
    fn test_select_last() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Type {
            element_type: "Button".to_string(),
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        let selected = select_elements(&matches, false, true, None, false).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].ref_id, "@e4");
    }

    #[test]
    fn test_select_nth() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Type {
            element_type: "Button".to_string(),
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        let selected = select_elements(&matches, false, false, Some(1), false).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].ref_id, "@e3");
    }

    #[test]
    fn test_select_all() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Type {
            element_type: "Button".to_string(),
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        let selected = select_elements(&matches, false, false, None, true).unwrap();
        assert_eq!(selected.len(), 3);
    }

    #[test]
    fn test_select_nth_out_of_range() {
        let snapshot = create_test_snapshot();
        let locator = FindLocator::Type {
            element_type: "Button".to_string(),
            action: None,
            action_value: None,
        };
        let matches = find_elements(&snapshot, &locator);
        let result = select_elements(&matches, false, false, Some(10), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_action_tap() {
        let action = FindAction::parse("tap", None).unwrap();
        assert!(matches!(action, FindAction::Tap));
    }

    #[test]
    fn test_parse_action_long_press() {
        let action = FindAction::parse("long-press", None).unwrap();
        assert!(matches!(action, FindAction::LongPress));
    }

    #[test]
    fn test_parse_action_fill() {
        let action = FindAction::parse("fill", Some("hello")).unwrap();
        assert!(matches!(action, FindAction::Fill(s) if s == "hello"));
    }

    #[test]
    fn test_parse_action_fill_requires_value() {
        let result = FindAction::parse("fill", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_action_clear() {
        let action = FindAction::parse("clear", None).unwrap();
        assert!(matches!(action, FindAction::Clear));
    }

    #[test]
    fn test_parse_action_unknown() {
        let result = FindAction::parse("unknown", None);
        assert!(result.is_err());
    }
}
