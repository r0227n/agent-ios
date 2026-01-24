//! Data types for UI snapshot representation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use agent_mobile_core::snapshot::Frame;

impl SnapshotElement {
    /// Check if element has displayable text (label, value, or placeholder).
    pub fn has_display_text(&self) -> bool {
        self.label.is_some() || self.value.is_some() || self.placeholder.is_some()
    }

    /// Check if element has visible content (text or is interactive).
    pub fn has_visible_content(&self) -> bool {
        self.has_display_text() || self.is_interactive
    }

    /// Check if element is an empty structure (no visible content).
    pub fn is_empty_structure(&self) -> bool {
        !self.has_visible_content()
    }

    /// Get display text with priority: label > value > placeholder.
    #[allow(dead_code)]
    pub fn display_text(&self) -> &str {
        self.label
            .as_deref()
            .or(self.value.as_deref())
            .or(self.placeholder.as_deref())
            .unwrap_or("-")
    }

    /// Check if text is contained in label or value (case-insensitive).
    pub fn contains_text(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.label
            .as_ref()
            .map(|l| l.to_lowercase().contains(&text_lower))
            .unwrap_or(false)
            || self
                .value
                .as_ref()
                .map(|v| v.to_lowercase().contains(&text_lower))
                .unwrap_or(false)
    }

    /// Check if text matches label or value exactly (case-insensitive).
    pub fn matches_text_exact(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.label
            .as_ref()
            .map(|l| l.to_lowercase() == text_lower)
            .unwrap_or(false)
            || self
                .value
                .as_ref()
                .map(|v| v.to_lowercase() == text_lower)
                .unwrap_or(false)
    }

    /// Check if label matches text (case-insensitive).
    pub fn matches_label(&self, text: &str, exact: bool) -> bool {
        let text_lower = text.to_lowercase();
        self.label
            .as_ref()
            .map(|l| {
                let label_lower = l.to_lowercase();
                if exact {
                    label_lower == text_lower
                } else {
                    label_lower.contains(&text_lower)
                }
            })
            .unwrap_or(false)
    }

    /// Check if placeholder matches text (case-insensitive).
    pub fn matches_placeholder(&self, text: &str, exact: bool) -> bool {
        let text_lower = text.to_lowercase();
        self.placeholder
            .as_ref()
            .map(|p| {
                let p_lower = p.to_lowercase();
                if exact {
                    p_lower == text_lower
                } else {
                    p_lower.contains(&text_lower)
                }
            })
            .unwrap_or(false)
    }
}

/// Complete UI snapshot with metadata and elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique snapshot identifier (e.g., "snap_abc12def")
    pub snapshot_id: String,

    /// Timestamp when snapshot was taken
    pub timestamp: DateTime<Utc>,

    /// Flattened list of all elements with refs
    pub elements: Vec<SnapshotElement>,
}

/// Single UI element with reference ID for AI agent interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotElement {
    /// Reference ID for quick interaction (e.g., "@e1", "@e2")
    #[serde(rename = "ref")]
    pub ref_id: String,

    /// Element type (e.g., "Button", "TextField", "StaticText")
    #[serde(rename = "type")]
    pub element_type: String,

    /// Element label/text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Element frame coordinates
    pub frame: Frame,

    /// Whether the element is enabled
    pub enabled: bool,

    /// Accessibility traits
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,

    /// Placeholder text (for text fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Current value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Child element indices (for tree structure)
    #[serde(skip, default)]
    pub children_indices: Vec<usize>,

    /// Depth in the tree (0 = root)
    #[serde(skip, default)]
    pub depth: u32,

    /// Whether the element is interactive/tappable
    #[serde(skip, default)]
    pub is_interactive: bool,

    /// Parent element index (None for root)
    #[serde(skip, default)]
    pub parent_index: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_element(
        label: Option<&str>,
        value: Option<&str>,
        placeholder: Option<&str>,
        is_interactive: bool,
    ) -> SnapshotElement {
        SnapshotElement {
            ref_id: "@e1".to_string(),
            element_type: "Button".to_string(),
            label: label.map(String::from),
            frame: Frame::zero(),
            enabled: true,
            traits: vec![],
            placeholder: placeholder.map(String::from),
            value: value.map(String::from),
            children_indices: vec![],
            depth: 0,
            is_interactive,
            parent_index: None,
        }
    }

    #[test]
    fn test_has_display_text() {
        assert!(test_element(Some("Label"), None, None, false).has_display_text());
        assert!(test_element(None, Some("Value"), None, false).has_display_text());
        assert!(test_element(None, None, Some("Placeholder"), false).has_display_text());
        assert!(!test_element(None, None, None, false).has_display_text());
    }

    #[test]
    fn test_has_visible_content() {
        assert!(test_element(Some("Label"), None, None, false).has_visible_content());
        assert!(test_element(None, None, None, true).has_visible_content());
        assert!(!test_element(None, None, None, false).has_visible_content());
    }

    #[test]
    fn test_is_empty_structure() {
        assert!(test_element(None, None, None, false).is_empty_structure());
        assert!(!test_element(Some("Label"), None, None, false).is_empty_structure());
        assert!(!test_element(None, None, None, true).is_empty_structure());
    }

    #[test]
    fn test_display_text() {
        assert_eq!(
            test_element(Some("Label"), None, None, false).display_text(),
            "Label"
        );
        assert_eq!(
            test_element(Some("Label"), Some("Value"), None, false).display_text(),
            "Label"
        );
        assert_eq!(
            test_element(None, Some("Value"), None, false).display_text(),
            "Value"
        );
        assert_eq!(
            test_element(None, None, Some("Placeholder"), false).display_text(),
            "Placeholder"
        );
        assert_eq!(test_element(None, None, None, false).display_text(), "-");
    }

    #[test]
    fn test_contains_text() {
        let elem = test_element(Some("Login Button"), None, None, false);
        assert!(elem.contains_text("Login"));
        assert!(elem.contains_text("login")); // case insensitive
        assert!(elem.contains_text("Button"));
        assert!(!elem.contains_text("Submit"));

        let elem_value = test_element(None, Some("test@example.com"), None, false);
        assert!(elem_value.contains_text("example"));
        assert!(elem_value.contains_text("EXAMPLE")); // case insensitive
    }

    #[test]
    fn test_matches_text_exact() {
        let elem = test_element(Some("Login"), None, None, false);
        assert!(elem.matches_text_exact("Login"));
        assert!(elem.matches_text_exact("login")); // case insensitive
        assert!(!elem.matches_text_exact("Log"));
        assert!(!elem.matches_text_exact("Login Button"));

        let elem_value = test_element(None, Some("Submit"), None, false);
        assert!(elem_value.matches_text_exact("Submit"));
        assert!(elem_value.matches_text_exact("SUBMIT"));
    }

    #[test]
    fn test_matches_label() {
        let elem = test_element(Some("Login Button"), None, None, false);
        assert!(elem.matches_label("Login Button", true));
        assert!(elem.matches_label("login button", true)); // case insensitive exact
        assert!(!elem.matches_label("Login", true)); // exact match fails

        assert!(elem.matches_label("Login", false)); // partial match
        assert!(elem.matches_label("Button", false));
        assert!(!elem.matches_label("Submit", false));
    }

    #[test]
    fn test_matches_placeholder() {
        let elem = test_element(None, None, Some("Enter your email"), false);
        assert!(elem.matches_placeholder("Enter your email", true));
        assert!(elem.matches_placeholder("enter your email", true)); // case insensitive
        assert!(!elem.matches_placeholder("Enter", true)); // exact match fails

        assert!(elem.matches_placeholder("email", false)); // partial match
        assert!(elem.matches_placeholder("Enter", false));
        assert!(!elem.matches_placeholder("password", false));
    }
}
