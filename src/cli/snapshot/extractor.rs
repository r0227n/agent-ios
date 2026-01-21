//! Extract RawElement tree from accessibility JSON.

use super::types::{Frame, RawElement};

/// Extract elements from iOS accessibility JSON (NESTED format).
pub fn extract_ios_elements(json: &serde_json::Value) -> Vec<RawElement> {
    let mut elements = Vec::new();

    // Handle both array and object root formats
    if let Some(arr) = json.as_array() {
        for item in arr {
            if let Some(elem) = extract_element(item) {
                elements.push(elem);
            }
        }
    } else if let Some(elem) = extract_element(json) {
        elements.push(elem);
    }

    elements
}

/// Extract a single element and its children recursively.
fn extract_element(node: &serde_json::Value) -> Option<RawElement> {
    let obj = node.as_object()?;

    // Get element type (remove "AX" prefix if present)
    let raw_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let element_type = normalize_ios_element_type(raw_type);

    // Skip Unknown types
    if element_type == "Unknown" {
        return None;
    }

    // Get label (AXLabel or AXValue)
    let label = obj
        .get("AXLabel")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("AXValue").and_then(|v| v.as_str()))
        .filter(|s| !s.is_empty())
        .map(String::from);

    // Get frame
    let frame = obj
        .get("frame")
        .and_then(|f| {
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
        })
        .unwrap_or_else(Frame::zero);

    // Get enabled state
    let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

    // Extract traits from type
    let traits = extract_traits(&element_type);

    // Get placeholder (for text fields)
    let placeholder = obj
        .get("AXPlaceholderValue")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Get value
    let value = obj
        .get("AXValue")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    // Extract children recursively
    let children = if let Some(children_arr) = obj.get("children").and_then(|c| c.as_array()) {
        children_arr.iter().filter_map(extract_element).collect()
    } else {
        Vec::new()
    };

    Some(RawElement {
        element_type,
        label,
        frame,
        enabled,
        traits,
        placeholder,
        value,
        children,
    })
}

/// Normalize iOS element type by removing "AX" prefix.
fn normalize_ios_element_type(raw_type: &str) -> String {
    if raw_type.starts_with("AX") {
        raw_type[2..].to_string()
    } else {
        raw_type.to_string()
    }
}

/// Extract traits based on element type.
fn extract_traits(element_type: &str) -> Vec<String> {
    let mut traits = Vec::new();

    match element_type {
        "Button" | "Link" | "MenuItem" | "MenuButton" => {
            traits.push("button".to_string());
        }
        "TextField" | "SecureTextField" | "SearchField" | "TextArea" => {
            traits.push("text_input".to_string());
        }
        "StaticText" => {
            traits.push("static_text".to_string());
        }
        "Image" => {
            traits.push("image".to_string());
        }
        "Switch" | "Checkbox" => {
            traits.push("toggle".to_string());
        }
        "Slider" => {
            traits.push("adjustable".to_string());
        }
        "ScrollView" | "Table" | "CollectionView" => {
            traits.push("scrollable".to_string());
        }
        "Tab" | "TabBar" => {
            traits.push("tab".to_string());
        }
        _ => {}
    }

    traits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_element() {
        let json = serde_json::json!({
            "type": "Button",
            "AXLabel": "Login",
            "frame": {"x": 100.0, "y": 200.0, "width": 80.0, "height": 44.0},
            "enabled": true
        });

        let elements = extract_ios_elements(&json);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].element_type, "Button");
        assert_eq!(elements[0].label, Some("Login".to_string()));
        assert!(elements[0].enabled);
    }

    #[test]
    fn test_extract_nested_elements() {
        let json = serde_json::json!({
            "type": "Window",
            "AXLabel": "Main",
            "frame": {"x": 0.0, "y": 0.0, "width": 390.0, "height": 844.0},
            "enabled": true,
            "children": [
                {
                    "type": "Button",
                    "AXLabel": "Submit",
                    "frame": {"x": 50.0, "y": 100.0, "width": 100.0, "height": 44.0},
                    "enabled": true
                },
                {
                    "type": "TextField",
                    "AXLabel": "Email",
                    "AXPlaceholderValue": "Enter email",
                    "frame": {"x": 20.0, "y": 200.0, "width": 350.0, "height": 44.0},
                    "enabled": true
                }
            ]
        });

        let elements = extract_ios_elements(&json);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].element_type, "Window");
        assert_eq!(elements[0].children.len(), 2);

        let button = &elements[0].children[0];
        assert_eq!(button.element_type, "Button");
        assert_eq!(button.label, Some("Submit".to_string()));

        let text_field = &elements[0].children[1];
        assert_eq!(text_field.element_type, "TextField");
        assert_eq!(text_field.placeholder, Some("Enter email".to_string()));
    }

    #[test]
    fn test_extract_array_format() {
        let json = serde_json::json!([
            {
                "type": "Application",
                "AXLabel": "MyApp",
                "frame": {"x": 0.0, "y": 0.0, "width": 402.0, "height": 874.0},
                "enabled": true,
                "children": []
            }
        ]);

        let elements = extract_ios_elements(&json);
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].element_type, "Application");
    }

    #[test]
    fn test_normalize_ax_prefix() {
        assert_eq!(normalize_ios_element_type("AXButton"), "Button");
        assert_eq!(normalize_ios_element_type("AXStaticText"), "StaticText");
        assert_eq!(normalize_ios_element_type("Button"), "Button");
    }

    #[test]
    fn test_extract_traits() {
        assert!(extract_traits("Button").contains(&"button".to_string()));
        assert!(extract_traits("TextField").contains(&"text_input".to_string()));
        assert!(extract_traits("ScrollView").contains(&"scrollable".to_string()));
    }
}
