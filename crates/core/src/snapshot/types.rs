//! Core types for UI snapshot representation.

use serde::{Deserialize, Serialize};

/// Element frame coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Frame {
    /// Calculate the center point of the frame.
    #[allow(dead_code)]
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Create a zero-sized frame at origin.
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Intermediate representation during extraction (before ref assignment).
#[derive(Debug, Clone)]
pub struct RawElement {
    pub element_type: String,
    pub label: Option<String>,
    pub frame: Frame,
    pub enabled: bool,
    pub traits: Vec<String>,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub children: Vec<RawElement>,
}

impl RawElement {
    /// Check if this element is interactive (tappable, typable, etc.).
    pub fn is_interactive(&self) -> bool {
        is_interactive_type(&self.element_type)
    }
}

/// Check if an element type is interactive.
pub fn is_interactive_type(element_type: &str) -> bool {
    matches!(
        element_type,
        // iOS interactive types
        "Button"
            | "Link"
            | "TextField"
            | "SecureTextField"
            | "SearchField"
            | "TextArea"
            | "Switch"
            | "Slider"
            | "Stepper"
            | "Picker"
            | "DatePicker"
            | "SegmentedControl"
            | "Tab"
            | "TabBar"
            | "MenuItem"
            | "MenuButton"
            | "PopUpButton"
            | "ComboBox"
            | "DisclosureTriangle"
            | "Checkbox"
            | "RadioButton"
            | "IncrementArrow"
            | "DecrementArrow"
            | "Cell"
            | "PageControl"
            // Android interactive types - Buttons
            | "ImageButton"
            | "FloatingActionButton"
            | "MaterialButton"
            // Android interactive types - Text input
            | "EditText"
            | "AutoCompleteTextView"
            | "TextInputEditText"
            | "TextInputLayout"
            // Android interactive types - Toggle/Selection
            | "ToggleButton"
            | "CheckBox"
            | "SwitchCompat"
            | "MaterialSwitch"
            | "CompoundButton"
            // Android interactive types - Adjustable
            | "SeekBar"
            | "RatingBar"
            // Android interactive types - Selection/Navigation
            | "Spinner"
            | "SearchView"
            | "TabItem"
            | "TabLayout"
            | "BottomNavigationView"
            | "NavigationBarView"
            // Android interactive types - Clickable containers (commonly used as buttons)
            | "CardView"
            | "MaterialCardView"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_center() {
        let frame = Frame {
            x: 100.0,
            y: 200.0,
            width: 50.0,
            height: 30.0,
        };
        assert_eq!(frame.center(), (125.0, 215.0));
    }

    #[test]
    fn test_frame_zero() {
        let frame = Frame::zero();
        assert_eq!(frame.x, 0.0);
        assert_eq!(frame.y, 0.0);
        assert_eq!(frame.width, 0.0);
        assert_eq!(frame.height, 0.0);
    }

    #[test]
    fn test_is_interactive_type() {
        assert!(is_interactive_type("Button"));
        assert!(is_interactive_type("TextField"));
        assert!(is_interactive_type("Link"));
        assert!(is_interactive_type("Switch"));
        assert!(!is_interactive_type("StaticText"));
        assert!(!is_interactive_type("Image"));
        assert!(!is_interactive_type("Window"));
    }
}
