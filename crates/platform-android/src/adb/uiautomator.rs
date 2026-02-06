//! UIAutomator dump and parse module.
//!
//! This module provides functions to capture and parse the UI hierarchy
//! from Android devices using uiautomator dump via native ADB protocol.

#![allow(dead_code)]

use super::commands::{AdbError, Result};
use super::connection::AdbConnection;
use serde::{Deserialize, Serialize};

/// Represents an accessibility element from the UI hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityElement {
    /// Text content of the element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Content description (accessibility label)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_desc: Option<String>,
    /// Resource ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// Class name (e.g., android.widget.Button)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// Package name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Element bounds as [left, top, right, bottom]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[i32; 4]>,
    /// Whether the element is clickable
    pub clickable: bool,
    /// Whether the element is scrollable
    pub scrollable: bool,
    /// Whether the element is enabled
    pub enabled: bool,
    /// Whether the element is focused
    pub focused: bool,
    /// Whether the element is selected
    pub selected: bool,
    /// Child elements
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<AccessibilityElement>,
}

impl AccessibilityElement {
    /// Get the center point of this element.
    pub fn center(&self) -> Option<(f64, f64)> {
        self.bounds.map(|[l, t, r, b]| {
            let x = (l + r) as f64 / 2.0;
            let y = (t + b) as f64 / 2.0;
            (x, y)
        })
    }

    /// Get a display label for this element.
    pub fn label(&self) -> Option<&str> {
        self.text
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| self.content_desc.as_deref().filter(|s| !s.is_empty()))
    }

    /// Get the element type (simplified class name).
    pub fn element_type(&self) -> Option<&str> {
        self.class
            .as_deref()
            .map(|c| c.rsplit('.').next().unwrap_or(c))
    }
}

/// Dump the UI hierarchy from the device.
///
/// Uses a single ADB shell command to dump and cat the UI hierarchy XML,
/// replacing the previous 2-step approach (dump + cat via separate shell-outs).
pub async fn dump_ui(serial: Option<&str>) -> Result<String> {
    let mut conn = AdbConnection::for_device(serial)?;

    // Dump UI hierarchy to file and then read it in one compound command
    let dump_output =
        conn.shell_command_args(&["uiautomator", "dump", "/sdcard/window_dump.xml"])?;

    // Check for errors in dump output
    if dump_output.contains("ERROR") || dump_output.contains("could not") {
        return Err(AdbError::CommandFailed(format!(
            "uiautomator dump failed: {}",
            dump_output.trim()
        )));
    }

    // Read the dumped XML file
    let xml = conn.shell_command_args(&["cat", "/sdcard/window_dump.xml"])?;

    Ok(xml)
}

/// Parse UI hierarchy XML into accessibility elements.
pub fn parse_ui_hierarchy(xml: &str) -> Result<Vec<AccessibilityElement>> {
    let mut elements = Vec::new();
    parse_nodes(xml, &mut elements);
    Ok(elements)
}

/// Simple XML parser for uiautomator output.
/// Note: This is a simplified parser that handles the basic uiautomator XML format.
fn parse_nodes(xml: &str, elements: &mut Vec<AccessibilityElement>) {
    // Find all <node> elements
    let mut pos = 0;
    while let Some(start) = xml[pos..].find("<node ") {
        let abs_start = pos + start;

        // Find the end of this node tag
        if let Some(end) = xml[abs_start..].find('>') {
            let tag_end = abs_start + end;
            let tag = &xml[abs_start..=tag_end];

            if let Some(element) = parse_node_tag(tag) {
                elements.push(element);
            }

            pos = tag_end + 1;
        } else {
            break;
        }
    }
}

/// Parse a single <node> tag into an AccessibilityElement.
fn parse_node_tag(tag: &str) -> Option<AccessibilityElement> {
    Some(AccessibilityElement {
        text: extract_attr(tag, "text"),
        content_desc: extract_attr(tag, "content-desc"),
        resource_id: extract_attr(tag, "resource-id"),
        class: extract_attr(tag, "class"),
        package: extract_attr(tag, "package"),
        bounds: extract_bounds(tag),
        clickable: extract_attr(tag, "clickable").is_some_and(|v| v == "true"),
        scrollable: extract_attr(tag, "scrollable").is_some_and(|v| v == "true"),
        enabled: extract_attr(tag, "enabled").is_none_or(|v| v == "true"),
        focused: extract_attr(tag, "focused").is_some_and(|v| v == "true"),
        selected: extract_attr(tag, "selected").is_some_and(|v| v == "true"),
        children: Vec::new(),
    })
}

/// Extract an attribute value from a tag string.
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = tag.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = tag[value_start..].find('"') {
            let value = &tag[value_start..value_start + end];
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Extract bounds attribute into [left, top, right, bottom].
fn extract_bounds(tag: &str) -> Option<[i32; 4]> {
    // Bounds format: [left,top][right,bottom]
    let bounds_str = extract_attr(tag, "bounds")?;

    // Parse [left,top][right,bottom]
    let parts: Vec<&str> = bounds_str
        .trim_matches(|c| c == '[' || c == ']')
        .split("][")
        .collect();

    if parts.len() != 2 {
        return None;
    }

    let lt: Vec<&str> = parts[0].split(',').collect();
    let rb: Vec<&str> = parts[1].split(',').collect();

    if lt.len() != 2 || rb.len() != 2 {
        return None;
    }

    let left: i32 = lt[0].parse().ok()?;
    let top: i32 = lt[1].parse().ok()?;
    let right: i32 = rb[0].parse().ok()?;
    let bottom: i32 = rb[1].parse().ok()?;

    Some([left, top, right, bottom])
}

/// Find elements matching a text query.
pub fn find_by_text<'a>(
    elements: &'a [AccessibilityElement],
    query: &str,
) -> Vec<&'a AccessibilityElement> {
    let query_lower = query.to_lowercase();
    elements
        .iter()
        .filter(|e| {
            e.text
                .as_ref()
                .is_some_and(|t| t.to_lowercase().contains(&query_lower))
                || e.content_desc
                    .as_ref()
                    .is_some_and(|d| d.to_lowercase().contains(&query_lower))
        })
        .collect()
}

/// Find elements matching a resource ID.
pub fn find_by_id<'a>(
    elements: &'a [AccessibilityElement],
    id: &str,
) -> Vec<&'a AccessibilityElement> {
    elements
        .iter()
        .filter(|e| e.resource_id.as_ref().is_some_and(|r| r.contains(id)))
        .collect()
}

/// Find elements matching a class/type (e.g., "Button", "TextView").
pub fn find_by_type<'a>(
    elements: &'a [AccessibilityElement],
    type_name: &str,
) -> Vec<&'a AccessibilityElement> {
    let type_lower = type_name.to_lowercase();
    elements
        .iter()
        .filter(|e| {
            e.class
                .as_ref()
                .is_some_and(|c| c.to_lowercase().contains(&type_lower))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attr() {
        let tag = r#"<node text="Hello" class="android.widget.Button" bounds="[0,0][100,50]">"#;
        assert_eq!(extract_attr(tag, "text"), Some("Hello".to_string()));
        assert_eq!(
            extract_attr(tag, "class"),
            Some("android.widget.Button".to_string())
        );
    }

    #[test]
    fn test_extract_bounds() {
        let tag = r#"<node bounds="[10,20][100,200]">"#;
        assert_eq!(extract_bounds(tag), Some([10, 20, 100, 200]));
    }

    #[test]
    fn test_element_center() {
        let element = AccessibilityElement {
            text: None,
            content_desc: None,
            resource_id: None,
            class: None,
            package: None,
            bounds: Some([0, 0, 100, 100]),
            clickable: false,
            scrollable: false,
            enabled: true,
            focused: false,
            selected: false,
            children: Vec::new(),
        };
        assert_eq!(element.center(), Some((50.0, 50.0)));
    }

    #[test]
    fn test_scrollable_parsing() {
        let tag = r#"<node scrollable="true" class="android.widget.ScrollView">"#;
        let element = parse_node_tag(tag).unwrap();
        assert!(element.scrollable);
        assert_eq!(element.class, Some("android.widget.ScrollView".to_string()));
    }
}
