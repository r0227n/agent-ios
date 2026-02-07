//! Extract RawElement tree from Android UIAutomator accessibility data.

use crate::adb::uiautomator::AccessibilityElement;
use agent_mobile_core::snapshot::{extract_traits_for_type, Frame, RawElement};

/// Extract elements from Android AccessibilityElement list.
///
/// Converts UIAutomator's AccessibilityElement structure into the common
/// RawElement format used for snapshot display.
pub fn extract_android_elements(elements: &[AccessibilityElement]) -> Vec<RawElement> {
    elements.iter().map(convert_element).collect()
}

/// Convert a single AccessibilityElement to RawElement.
fn convert_element(element: &AccessibilityElement) -> RawElement {
    // Extract element type from class name (e.g., "android.widget.Button" -> "Button")
    let element_type = element
        .element_type()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "View".to_string());

    // Get label from text or content-desc
    let label = element.label().map(|s| s.to_string());

    // Convert bounds to Frame
    let frame = element
        .bounds
        .map(|[left, top, right, bottom]| Frame {
            x: left as f64,
            y: top as f64,
            width: (right - left) as f64,
            height: (bottom - top) as f64,
        })
        .unwrap_or_else(Frame::zero);

    // Extract traits from type
    let traits = extract_traits_for_type(&element_type);

    // Add additional traits based on Android-specific attributes
    let mut all_traits = traits;
    if element.scrollable && !all_traits.iter().any(|t| t == "scrollable") {
        all_traits.push("scrollable".to_string());
    }
    if element.clickable
        && !all_traits.iter().any(|t| t == "button")
        && !all_traits.iter().any(|t| t == "text_input")
        && !all_traits.iter().any(|t| t == "clickable")
    {
        all_traits.push("clickable".to_string());
    }

    // Convert children recursively
    let children = element.children.iter().map(convert_element).collect();

    RawElement {
        element_type,
        label,
        frame,
        enabled: element.enabled,
        traits: all_traits,
        placeholder: None, // Android doesn't have a direct placeholder equivalent in UIAutomator
        value: None,       // Could be extracted from text for certain element types
        children,
    }
}

/// Normalize Android class name to element type.
///
/// Extracts the simple class name from fully qualified Android class names.
/// e.g., "android.widget.Button" -> "Button"
#[allow(dead_code)]
pub fn normalize_android_element_type(class_name: &str) -> String {
    class_name
        .rsplit('.')
        .next()
        .unwrap_or(class_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_accessibility_element(
        class: &str,
        text: Option<&str>,
        content_desc: Option<&str>,
        bounds: Option<[i32; 4]>,
        clickable: bool,
        scrollable: bool,
        children: Vec<AccessibilityElement>,
    ) -> AccessibilityElement {
        AccessibilityElement {
            text: text.map(String::from),
            content_desc: content_desc.map(String::from),
            resource_id: None,
            class: Some(class.to_string()),
            package: Some("com.example.app".to_string()),
            bounds,
            clickable,
            scrollable,
            enabled: true,
            focused: false,
            selected: false,
            children,
        }
    }

    #[test]
    fn test_convert_simple_button() {
        let element = make_accessibility_element(
            "android.widget.Button",
            Some("Login"),
            None,
            Some([100, 200, 300, 250]),
            true,
            false,
            vec![],
        );

        let raw = convert_element(&element);

        assert_eq!(raw.element_type, "Button");
        assert_eq!(raw.label, Some("Login".to_string()));
        assert_eq!(raw.frame.x, 100.0);
        assert_eq!(raw.frame.y, 200.0);
        assert_eq!(raw.frame.width, 200.0);
        assert_eq!(raw.frame.height, 50.0);
        assert!(raw.traits.contains(&"button".to_string()));
    }

    #[test]
    fn test_convert_text_input() {
        let element = make_accessibility_element(
            "android.widget.EditText",
            None,
            Some("Email input"),
            Some([50, 100, 350, 150]),
            true,
            false,
            vec![],
        );

        let raw = convert_element(&element);

        assert_eq!(raw.element_type, "EditText");
        assert_eq!(raw.label, Some("Email input".to_string()));
        assert!(raw.traits.contains(&"text_input".to_string()));
    }

    #[test]
    fn test_convert_scrollable_list() {
        let element = make_accessibility_element(
            "androidx.recyclerview.widget.RecyclerView",
            None,
            None,
            Some([0, 200, 1080, 1800]),
            false,
            true,
            vec![],
        );

        let raw = convert_element(&element);

        assert_eq!(raw.element_type, "RecyclerView");
        assert!(raw.traits.contains(&"scrollable".to_string()));
    }

    #[test]
    fn test_convert_nested_elements() {
        let child = make_accessibility_element(
            "android.widget.Button",
            Some("Submit"),
            None,
            Some([100, 300, 200, 350]),
            true,
            false,
            vec![],
        );

        let parent = make_accessibility_element(
            "android.widget.LinearLayout",
            None,
            None,
            Some([0, 0, 1080, 1920]),
            false,
            false,
            vec![child],
        );

        let elements = extract_android_elements(&[parent]);

        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].element_type, "LinearLayout");
        assert_eq!(elements[0].children.len(), 1);
        assert_eq!(elements[0].children[0].element_type, "Button");
        assert_eq!(elements[0].children[0].label, Some("Submit".to_string()));
    }

    #[test]
    fn test_normalize_android_element_type() {
        assert_eq!(
            normalize_android_element_type("android.widget.Button"),
            "Button"
        );
        assert_eq!(
            normalize_android_element_type("android.widget.TextView"),
            "TextView"
        );
        assert_eq!(
            normalize_android_element_type("androidx.recyclerview.widget.RecyclerView"),
            "RecyclerView"
        );
        assert_eq!(normalize_android_element_type("Button"), "Button");
    }

    #[test]
    fn test_clickable_trait_added() {
        // A FrameLayout that is clickable but not a button type
        let element = make_accessibility_element(
            "android.widget.FrameLayout",
            Some("Card"),
            None,
            Some([0, 0, 300, 200]),
            true, // clickable
            false,
            vec![],
        );

        let raw = convert_element(&element);

        assert_eq!(raw.element_type, "FrameLayout");
        assert!(raw.traits.contains(&"clickable".to_string()));
    }

    #[test]
    fn test_button_no_duplicate_clickable() {
        // A Button that is clickable should not have both "button" and "clickable"
        let element = make_accessibility_element(
            "android.widget.Button",
            Some("OK"),
            None,
            Some([0, 0, 100, 50]),
            true,
            false,
            vec![],
        );

        let raw = convert_element(&element);

        assert!(raw.traits.contains(&"button".to_string()));
        assert!(!raw.traits.contains(&"clickable".to_string()));
    }

    #[test]
    fn test_label_prefers_text_over_content_desc() {
        let element = make_accessibility_element(
            "android.widget.Button",
            Some("Primary Text"),
            Some("Accessibility Label"),
            Some([0, 0, 100, 50]),
            true,
            false,
            vec![],
        );

        let raw = convert_element(&element);

        // text is preferred
        assert_eq!(raw.label, Some("Primary Text".to_string()));
    }

    #[test]
    fn test_label_falls_back_to_content_desc() {
        let element = make_accessibility_element(
            "android.widget.ImageButton",
            None,
            Some("Menu"),
            Some([0, 0, 50, 50]),
            true,
            false,
            vec![],
        );

        let raw = convert_element(&element);

        assert_eq!(raw.label, Some("Menu".to_string()));
    }
}
