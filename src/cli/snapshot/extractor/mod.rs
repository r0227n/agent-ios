//! Common extraction utilities for UI elements.
//!
//! This module provides shared functionality for extracting and processing
//! UI elements from both iOS and Android platforms.

mod extractor_android;
mod extractor_ios;

pub use extractor_android::extract_android_elements;
pub use extractor_ios::extract_ios_elements;

/// Extract traits based on element type.
///
/// This function maps UI element types to semantic traits that describe
/// the element's behavior. Works for both iOS and Android element types.
pub fn extract_traits_for_type(element_type: &str) -> Vec<String> {
    let mut traits = Vec::new();

    match element_type {
        // iOS Button types
        "Button" | "Link" | "MenuItem" | "MenuButton" => {
            traits.push("button".to_string());
        }
        // Android Button types
        "ImageButton" | "FloatingActionButton" | "MaterialButton" => {
            traits.push("button".to_string());
        }

        // iOS Text input types
        "TextField" | "SecureTextField" | "SearchField" | "TextArea" => {
            traits.push("text_input".to_string());
        }
        // Android Text input types
        "EditText" | "AutoCompleteTextView" | "TextInputEditText" | "TextInputLayout" => {
            traits.push("text_input".to_string());
        }

        // iOS Static text
        "StaticText" => {
            traits.push("static_text".to_string());
        }
        // Android Text views
        "TextView" => {
            traits.push("static_text".to_string());
        }

        // iOS Image
        "Image" => {
            traits.push("image".to_string());
        }
        // Android Image views
        "ImageView" => {
            traits.push("image".to_string());
        }

        // iOS Toggle types
        "Switch" | "Checkbox" => {
            traits.push("toggle".to_string());
        }
        // Android Toggle types
        "ToggleButton" | "CheckBox" | "RadioButton" | "CompoundButton" | "SwitchCompat"
        | "MaterialSwitch" => {
            traits.push("toggle".to_string());
        }

        // iOS Adjustable types
        "Slider" => {
            traits.push("adjustable".to_string());
        }
        // Android Adjustable types
        "SeekBar" | "RatingBar" | "ProgressBar" => {
            traits.push("adjustable".to_string());
        }

        // iOS Scrollable types
        "ScrollView" | "Table" | "CollectionView" => {
            traits.push("scrollable".to_string());
        }
        // Android Scrollable types
        "RecyclerView" | "ListView" | "GridView" | "NestedScrollView" | "HorizontalScrollView" => {
            traits.push("scrollable".to_string());
        }

        // iOS Tab types
        "Tab" | "TabBar" => {
            traits.push("tab".to_string());
        }
        // Android Tab/Navigation types
        "TabItem" | "TabLayout" | "BottomNavigationView" | "NavigationBarView" => {
            traits.push("tab".to_string());
        }

        // Android Spinner/Dropdown
        "Spinner" | "SearchView" => {
            traits.push("selectable".to_string());
        }

        _ => {}
    }

    traits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_button_traits() {
        assert!(extract_traits_for_type("Button").contains(&"button".to_string()));
        assert!(extract_traits_for_type("Link").contains(&"button".to_string()));
        assert!(extract_traits_for_type("MenuItem").contains(&"button".to_string()));
    }

    #[test]
    fn test_android_button_traits() {
        assert!(extract_traits_for_type("ImageButton").contains(&"button".to_string()));
        assert!(extract_traits_for_type("FloatingActionButton").contains(&"button".to_string()));
    }

    #[test]
    fn test_ios_text_input_traits() {
        assert!(extract_traits_for_type("TextField").contains(&"text_input".to_string()));
        assert!(extract_traits_for_type("SecureTextField").contains(&"text_input".to_string()));
    }

    #[test]
    fn test_android_text_input_traits() {
        assert!(extract_traits_for_type("EditText").contains(&"text_input".to_string()));
        assert!(extract_traits_for_type("TextInputEditText").contains(&"text_input".to_string()));
    }

    #[test]
    fn test_scrollable_traits() {
        // iOS
        assert!(extract_traits_for_type("ScrollView").contains(&"scrollable".to_string()));
        assert!(extract_traits_for_type("Table").contains(&"scrollable".to_string()));
        // Android
        assert!(extract_traits_for_type("RecyclerView").contains(&"scrollable".to_string()));
        assert!(extract_traits_for_type("ListView").contains(&"scrollable".to_string()));
    }

    #[test]
    fn test_toggle_traits() {
        // iOS
        assert!(extract_traits_for_type("Switch").contains(&"toggle".to_string()));
        assert!(extract_traits_for_type("Checkbox").contains(&"toggle".to_string()));
        // Android
        assert!(extract_traits_for_type("CheckBox").contains(&"toggle".to_string()));
        assert!(extract_traits_for_type("ToggleButton").contains(&"toggle".to_string()));
    }

    #[test]
    fn test_tab_traits() {
        // iOS
        assert!(extract_traits_for_type("Tab").contains(&"tab".to_string()));
        assert!(extract_traits_for_type("TabBar").contains(&"tab".to_string()));
        // Android
        assert!(extract_traits_for_type("TabItem").contains(&"tab".to_string()));
        assert!(extract_traits_for_type("BottomNavigationView").contains(&"tab".to_string()));
    }

    #[test]
    fn test_unknown_type_has_no_traits() {
        assert!(extract_traits_for_type("UnknownWidget").is_empty());
        assert!(extract_traits_for_type("View").is_empty());
    }
}
