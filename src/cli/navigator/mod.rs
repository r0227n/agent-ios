//! Navigator command implementation.
//!
//! Provides cross-platform element search and interaction capabilities.
//! Find elements by text, type, or ID, then perform actions on them.

use clap::Args;
use serde::Serialize;

use crate::cli::helpers::{CommandResult, OutputFormat};
use crate::types::Platform;

/// Navigator command arguments.
#[derive(Args, Debug)]
pub struct NavigatorArgs {
    /// Find element by text (partial match).
    #[arg(long)]
    pub find: Option<String>,

    /// Find element by type (e.g., Button, TextField).
    #[arg(long)]
    pub find_type: Option<String>,

    /// Find element by ID (Android resource-id, iOS identifier).
    #[arg(long)]
    pub find_id: Option<String>,

    /// Tap the found element.
    #[arg(long)]
    pub tap: bool,

    /// Enter text into the found element.
    #[arg(long)]
    pub enter_text: Option<String>,

    /// List all elements.
    #[arg(long)]
    pub list: bool,

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

/// Found element result.
#[derive(Debug, Serialize)]
pub struct FoundElement {
    pub label: String,
    pub element_type: String,
    pub center: Option<(f64, f64)>,
    pub clickable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
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

/// Execute the navigator command.
pub async fn run(args: NavigatorArgs) -> CommandResult {
    let platform = match &args.platform {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?,
        None => detect_platform().await?,
    };

    if args.list {
        return execute_list(platform, &args).await;
    }

    if args.find.is_some() || args.find_type.is_some() || args.find_id.is_some() {
        return execute_find(platform, &args).await;
    }

    Err("No action specified. Use --find, --find-type, --find-id, or --list.".into())
}

/// Execute list all elements.
async fn execute_list(platform: Platform, args: &NavigatorArgs) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                let json_str = client.accessibility_info(None, true).await?;
                let json: serde_json::Value = serde_json::from_str(&json_str)?;
                let elements = extract_ios_elements(&json);

                if args.output.is_json() {
                    println!("{}", serde_json::to_string_pretty(&elements)?);
                } else {
                    print_elements(&elements);
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let xml = uiautomator::dump_ui(args.udid.as_deref()).await?;
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;
            let found_elements = convert_android_elements(&elements);

            if args.output.is_json() {
                println!("{}", serde_json::to_string_pretty(&found_elements)?);
            } else {
                print_elements(&found_elements);
            }
            Ok(())
        }
    }
}

/// Execute find and optionally interact.
async fn execute_find(platform: Platform, args: &NavigatorArgs) -> CommandResult {
    match platform {
        Platform::Ios => execute_find_ios(args).await,
        Platform::Android => execute_find_android(args).await,
    }
}

/// Find element on iOS.
async fn execute_find_ios(args: &NavigatorArgs) -> CommandResult {
    use crate::cli::helpers::with_client;
    use crate::cli::idb::hid::events;

    with_client(args.udid.as_deref(), |mut client| async move {
        let json_str = client.accessibility_info(None, true).await?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;
        let all_elements = extract_ios_elements(&json);

        // Find matching elements
        let found = find_matching_elements(
            &all_elements,
            args.find.as_deref(),
            args.find_type.as_deref(),
            args.find_id.as_deref(),
        );

        if found.is_empty() {
            return Err("No elements found matching the criteria.".into());
        }

        // If tap or enter_text, perform action on first match
        if args.tap || args.enter_text.is_some() {
            let element = &found[0];
            if let Some((x, y)) = element.center {
                // Tap the element
                let tap_events = events::tap_to_events(x, y, None);
                client.hid(tap_events).await?;

                // If enter_text, also input text
                if let Some(text) = &args.enter_text {
                    // Small delay before typing
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    let text_events = events::text_to_events(text)?;
                    client.hid(text_events).await?;
                }

                println!(
                    "Tapped element: \"{}\" at ({:.0}, {:.0})",
                    element.label, x, y
                );
            } else {
                return Err("Found element has no coordinates.".into());
            }
        } else {
            // Just print found elements
            if args.output.is_json() {
                println!("{}", serde_json::to_string_pretty(&found)?);
            } else {
                print_found_elements(&found);
            }
        }

        Ok(())
    })
    .await
}

/// Find element on Android.
async fn execute_find_android(args: &NavigatorArgs) -> CommandResult {
    use crate::platform::android::adb::{input, uiautomator};

    let xml = uiautomator::dump_ui(args.udid.as_deref()).await?;
    let all_elements = uiautomator::parse_ui_hierarchy(&xml)?;
    let converted = convert_android_elements(&all_elements);

    // Find matching elements
    let found = find_matching_elements(
        &converted,
        args.find.as_deref(),
        args.find_type.as_deref(),
        args.find_id.as_deref(),
    );

    if found.is_empty() {
        return Err("No elements found matching the criteria.".into());
    }

    // If tap or enter_text, perform action on first match
    if args.tap || args.enter_text.is_some() {
        let element = &found[0];
        if let Some((x, y)) = element.center {
            // Tap the element
            input::tap(args.udid.as_deref(), x, y).await?;

            // If enter_text, also input text
            if let Some(text) = &args.enter_text {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                input::text(args.udid.as_deref(), text).await?;
            }

            println!(
                "Tapped element: \"{}\" at ({:.0}, {:.0})",
                element.label, x, y
            );
        } else {
            return Err("Found element has no coordinates.".into());
        }
    } else {
        // Just print found elements
        if args.output.is_json() {
            println!("{}", serde_json::to_string_pretty(&found)?);
        } else {
            print_found_elements(&found);
        }
    }

    Ok(())
}

/// Extract elements from iOS accessibility JSON.
fn extract_ios_elements(json: &serde_json::Value) -> Vec<FoundElement> {
    let mut elements = Vec::new();

    fn traverse(node: &serde_json::Value, elements: &mut Vec<FoundElement>) {
        // 配列の場合は各要素を再帰処理
        if let Some(arr) = node.as_array() {
            for item in arr {
                traverse(item, elements);
            }
            return;
        }

        if let Some(obj) = node.as_object() {
            // 正しいキー: "type" (not "AXRole")
            let element_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let label = obj
                .get("AXLabel")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("AXValue").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            // 正しいキー: "AXUniqueId" (not "AXIdentifier")
            let identifier = obj
                .get("AXUniqueId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // 正しいキー: "frame" (not "AXFrame")
            let frame = obj.get("frame").and_then(|f| {
                let x = f.get("x")?.as_f64()?;
                let y = f.get("y")?.as_f64()?;
                let w = f.get("width")?.as_f64()?;
                let h = f.get("height")?.as_f64()?;
                Some((x + w / 2.0, y + h / 2.0))
            });

            let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            // 有効な要素タイプがあれば追加
            if !element_type.is_empty() && element_type != "Unknown" {
                elements.push(FoundElement {
                    label,
                    element_type,
                    center: frame,
                    clickable: enabled,
                    resource_id: identifier,
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

/// Convert Android elements to FoundElement.
fn convert_android_elements(
    elements: &[crate::platform::android::adb::uiautomator::AccessibilityElement],
) -> Vec<FoundElement> {
    elements
        .iter()
        .map(|e| FoundElement {
            label: e.label().unwrap_or("").to_string(),
            element_type: e.element_type().unwrap_or("Unknown").to_string(),
            center: e.center(),
            clickable: e.clickable,
            resource_id: e.resource_id.clone(),
        })
        .collect()
}

/// Find elements matching criteria.
fn find_matching_elements<'a>(
    elements: &'a [FoundElement],
    text: Option<&str>,
    type_name: Option<&str>,
    id: Option<&str>,
) -> Vec<&'a FoundElement> {
    elements
        .iter()
        .filter(|e| {
            let text_match =
                text.map_or(true, |t| e.label.to_lowercase().contains(&t.to_lowercase()));
            let type_match = type_name.map_or(true, |t| {
                e.element_type.to_lowercase().contains(&t.to_lowercase())
            });
            let id_match = id.map_or(true, |i| {
                e.resource_id
                    .as_ref()
                    .map_or(false, |r| r.to_lowercase().contains(&i.to_lowercase()))
            });
            text_match && type_match && id_match
        })
        .collect()
}

/// Print elements in human-readable format (Python版と同じフォーマット).
fn print_elements(elements: &[FoundElement]) {
    println!("Tappable elements ({}):", elements.len());
    for e in elements.iter().take(30) {
        if let Some((x, y)) = e.center {
            if e.label.is_empty() {
                println!("  {}: ({:.0}, {:.0})", e.element_type, x, y);
            } else {
                println!("  {}: \"{}\" ({:.0}, {:.0})", e.element_type, e.label, x, y);
            }
        } else if e.label.is_empty() {
            println!("  {}", e.element_type);
        } else {
            println!("  {}: \"{}\"", e.element_type, e.label);
        }
    }
    if elements.len() > 30 {
        println!("  ... and {} more", elements.len() - 30);
    }
}

/// Print found elements.
fn print_found_elements(elements: &[&FoundElement]) {
    println!("Found {} element(s):", elements.len());
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
        if let Some(id) = &e.resource_id {
            println!("   ID: {}", id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ios_elements_nested_format() {
        let json = serde_json::json!({
            "type": "Window",
            "AXLabel": "Test App",
            "frame": {"x": 0.0, "y": 0.0, "width": 390.0, "height": 844.0},
            "enabled": true,
            "children": [{
                "type": "Button",
                "AXLabel": "Login",
                "AXUniqueId": "loginButton",
                "frame": {"x": 50.0, "y": 100.0, "width": 100.0, "height": 44.0},
                "enabled": true
            }, {
                "type": "TextField",
                "AXLabel": "Username",
                "AXValue": "user@example.com",
                "frame": {"x": 20.0, "y": 200.0, "width": 350.0, "height": 44.0},
                "enabled": true
            }]
        });

        let elements = extract_ios_elements(&json);

        assert_eq!(elements.len(), 3);

        // Window element
        assert_eq!(elements[0].element_type, "Window");
        assert_eq!(elements[0].label, "Test App");
        assert_eq!(elements[0].center, Some((195.0, 422.0)));

        // Button element
        assert_eq!(elements[1].element_type, "Button");
        assert_eq!(elements[1].label, "Login");
        assert_eq!(elements[1].resource_id, Some("loginButton".to_string()));
        assert_eq!(elements[1].center, Some((100.0, 122.0)));

        // TextField element
        assert_eq!(elements[2].element_type, "TextField");
        assert_eq!(elements[2].label, "Username");
    }

    #[test]
    fn test_extract_ios_elements_empty_json() {
        let json = serde_json::json!({});
        let elements = extract_ios_elements(&json);
        assert!(elements.is_empty());
    }

    #[test]
    fn test_extract_ios_elements_unknown_type_filtered() {
        let json = serde_json::json!({
            "AXLabel": "Orphan Label"
        });
        let elements = extract_ios_elements(&json);
        assert!(elements.is_empty());
    }

    #[test]
    fn test_extract_ios_elements_with_axvalue() {
        let json = serde_json::json!({
            "type": "StaticText",
            "AXValue": "Some static text",
            "frame": {"x": 10.0, "y": 10.0, "width": 100.0, "height": 20.0}
        });

        let elements = extract_ios_elements(&json);

        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].element_type, "StaticText");
        assert_eq!(elements[0].label, "Some static text");
    }

    #[test]
    fn test_extract_ios_elements_disabled() {
        let json = serde_json::json!({
            "type": "Button",
            "AXLabel": "Disabled Button",
            "enabled": false,
            "frame": {"x": 0.0, "y": 0.0, "width": 100.0, "height": 44.0}
        });

        let elements = extract_ios_elements(&json);

        assert_eq!(elements.len(), 1);
        assert!(!elements[0].clickable);
    }

    #[test]
    fn test_find_matching_elements_by_text() {
        let elements = vec![
            FoundElement {
                label: "Login Button".to_string(),
                element_type: "Button".to_string(),
                center: Some((100.0, 100.0)),
                clickable: true,
                resource_id: None,
            },
            FoundElement {
                label: "Logout Button".to_string(),
                element_type: "Button".to_string(),
                center: Some((100.0, 200.0)),
                clickable: true,
                resource_id: None,
            },
        ];

        let found = find_matching_elements(&elements, Some("login"), None, None);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].label, "Login Button");
    }

    #[test]
    fn test_find_matching_elements_by_type() {
        let elements = vec![
            FoundElement {
                label: "Text".to_string(),
                element_type: "StaticText".to_string(),
                center: Some((100.0, 100.0)),
                clickable: false,
                resource_id: None,
            },
            FoundElement {
                label: "Click me".to_string(),
                element_type: "Button".to_string(),
                center: Some((100.0, 200.0)),
                clickable: true,
                resource_id: None,
            },
        ];

        let found = find_matching_elements(&elements, None, Some("button"), None);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].element_type, "Button");
    }

    #[test]
    fn test_find_matching_elements_by_id() {
        let elements = vec![
            FoundElement {
                label: "Submit".to_string(),
                element_type: "Button".to_string(),
                center: Some((100.0, 100.0)),
                clickable: true,
                resource_id: Some("submitBtn".to_string()),
            },
            FoundElement {
                label: "Cancel".to_string(),
                element_type: "Button".to_string(),
                center: Some((100.0, 200.0)),
                clickable: true,
                resource_id: Some("cancelBtn".to_string()),
            },
        ];

        let found = find_matching_elements(&elements, None, None, Some("submit"));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].resource_id, Some("submitBtn".to_string()));
    }

    #[test]
    fn test_extract_ios_elements_array_format() {
        // gRPCレスポンスの実際の形式: ルートが配列
        let json = serde_json::json!([
            {
                "type": "Application",
                "AXLabel": " ",
                "frame": {"x": 0.0, "y": 0.0, "width": 402.0, "height": 874.0},
                "enabled": true,
                "children": [
                    {
                        "type": "Button",
                        "AXLabel": "マップ",
                        "AXUniqueId": "マップ",
                        "frame": {"x": 24.3, "y": 88.0, "width": 168.3, "height": 191.0},
                        "enabled": true,
                        "children": []
                    },
                    {
                        "type": "Button",
                        "AXLabel": "カレンダー",
                        "frame": {"x": 209.3, "y": 88.0, "width": 168.3, "height": 191.0},
                        "enabled": true,
                        "children": []
                    }
                ]
            }
        ]);

        let elements = extract_ios_elements(&json);

        // Application + 2 Buttons = 3 elements
        assert_eq!(elements.len(), 3);

        // Application element
        assert_eq!(elements[0].element_type, "Application");
        assert_eq!(elements[0].label, " ");
        assert_eq!(elements[0].center, Some((201.0, 437.0)));

        // Button elements
        assert_eq!(elements[1].element_type, "Button");
        assert_eq!(elements[1].label, "マップ");
        assert_eq!(elements[1].resource_id, Some("マップ".to_string()));

        assert_eq!(elements[2].element_type, "Button");
        assert_eq!(elements[2].label, "カレンダー");
    }

    #[test]
    fn test_extract_ios_elements_empty_array() {
        let json = serde_json::json!([]);
        let elements = extract_ios_elements(&json);
        assert!(elements.is_empty());
    }
}
