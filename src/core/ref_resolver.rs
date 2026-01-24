//! Ref Resolution System
//!
//! System for converting ref identifiers in @e1, @e2 format to coordinates.

use agent_mobile_core::snapshot::Frame;

use crate::helpers::client::CommandResult;
use crate::snapshot::types::{Snapshot, SnapshotElement};

/// UI Element Target type for Core Commands
#[derive(Debug, Clone)]
pub enum ElementTarget {
    /// Element reference from snapshot (e.g., "@e1", "@e42")
    Ref(String),
    /// Text to search for in snapshot
    Text(String),
    /// Direct coordinates (x, y)
    Coords(f64, f64),
    /// Special key (e.g., "home", "back", "enter")
    Key(String),
    /// Special position (e.g., "center")
    Position(String),
}

/// Resolved element with coordinates
#[derive(Debug, Clone)]
pub struct ResolvedElement {
    pub ref_id: String,
    pub element_type: String,
    pub label: Option<String>,
    pub frame: Frame,
    pub enabled: bool,
    pub value: Option<String>,
    pub traits: Vec<String>,
    pub placeholder: Option<String>,
    pub depth: u32,
    pub is_interactive: bool,
}

impl ResolvedElement {
    /// Get the center coordinates of the element
    pub fn center(&self) -> (f64, f64) {
        self.frame.center()
    }

    /// Check if this element is a text input
    pub fn is_text_input(&self) -> bool {
        matches!(
            self.element_type.as_str(),
            "TextField"
                | "SecureTextField"
                | "SearchField"
                | "TextArea"
                | "EditText"
                | "AutoCompleteTextView"
                | "TextInputEditText"
                | "TextInputLayout"
        )
    }

    /// Create a synthetic element from coordinates
    pub fn from_coords(x: f64, y: f64) -> Self {
        Self {
            ref_id: format!("coords({},{})", x, y),
            element_type: "Coordinate".to_string(),
            label: None,
            frame: Frame {
                x,
                y,
                width: 0.0,
                height: 0.0,
            },
            enabled: true,
            value: None,
            traits: vec![],
            placeholder: None,
            depth: 0,
            is_interactive: false,
        }
    }
}

impl ElementTarget {
    /// Parse a target string into an ElementTarget type
    pub fn parse(s: &str) -> Self {
        let s = s.trim();

        // Check for @eN reference format
        if s.starts_with('@') {
            return ElementTarget::Ref(s.to_string());
        }

        // Check for coordinate format (x,y)
        if let Some((x, y)) = parse_coords(s) {
            return ElementTarget::Coords(x, y);
        }

        // Check for special positions (e.g., "center")
        if is_special_position(s) {
            return ElementTarget::Position(s.to_lowercase());
        }

        // Check for special keys
        if is_special_key(s) {
            return ElementTarget::Key(s.to_lowercase());
        }

        // Treat as text to search
        ElementTarget::Text(s.to_string())
    }

    /// Check if this target is a special key
    #[allow(dead_code)]
    pub fn is_key(&self) -> bool {
        matches!(self, ElementTarget::Key(_))
    }
}

/// Parse coordinate string "x,y" into (x, y)
fn parse_coords(s: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return None;
    }
    let x: f64 = parts[0].trim().parse().ok()?;
    let y: f64 = parts[1].trim().parse().ok()?;
    Some((x, y))
}

/// Check if a string is a special position
fn is_special_position(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "center")
}

/// Check if a string is a special key
fn is_special_key(s: &str) -> bool {
    matches!(
        s.to_lowercase().as_str(),
        "home"
            | "back"
            | "enter"
            | "return"
            | "tab"
            | "escape"
            | "esc"
            | "delete"
            | "backspace"
            | "space"
            | "up"
            | "down"
            | "left"
            | "right"
            | "lock"
            | "power"
            | "siri"
            | "menu"
            | "volume-up"
            | "volume-down"
            | "volume_up"
            | "volume_down"
    )
}

/// Find an element by ref in a snapshot
pub fn find_by_ref<'a>(snapshot: &'a Snapshot, ref_id: &str) -> Option<&'a SnapshotElement> {
    snapshot.elements.iter().find(|e| e.ref_id == ref_id)
}

/// Find an element by text (label or value) in a snapshot.
///
/// Search priority:
/// 1. Exact label match (case-sensitive)
/// 2. Exact value match (case-sensitive)
/// 3. Exact label match (case-insensitive)
/// 4. Partial label match (case-insensitive)
pub fn find_by_text<'a>(snapshot: &'a Snapshot, text: &str) -> Option<&'a SnapshotElement> {
    // First try exact match on label (case-sensitive)
    snapshot
        .elements
        .iter()
        .find(|e| e.label.as_ref().map(|l| l == text).unwrap_or(false))
        // Then try exact match on value (case-sensitive)
        .or_else(|| {
            snapshot
                .elements
                .iter()
                .find(|e| e.value.as_ref().map(|v| v == text).unwrap_or(false))
        })
        // Then try case-insensitive exact match on label
        .or_else(|| {
            snapshot
                .elements
                .iter()
                .find(|e| e.matches_label(text, true))
        })
        // Finally try partial match on label (contains)
        .or_else(|| snapshot.elements.iter().find(|e| e.contains_text(text)))
}

/// Convert a SnapshotElement to a ResolvedElement
pub fn to_resolved(elem: &SnapshotElement) -> ResolvedElement {
    ResolvedElement {
        ref_id: elem.ref_id.clone(),
        element_type: elem.element_type.clone(),
        label: elem.label.clone(),
        frame: elem.frame.clone(),
        enabled: elem.enabled,
        value: elem.value.clone(),
        traits: elem.traits.clone(),
        placeholder: elem.placeholder.clone(),
        depth: elem.depth,
        is_interactive: elem.is_interactive,
    }
}

/// Resolve a target to an element from a snapshot
pub fn resolve_from_snapshot(
    snapshot: &Snapshot,
    target: &ElementTarget,
) -> CommandResult<ResolvedElement> {
    match target {
        ElementTarget::Coords(x, y) => Ok(ResolvedElement::from_coords(*x, *y)),
        ElementTarget::Ref(ref_id) => {
            let elem = find_by_ref(snapshot, ref_id).ok_or_else(|| {
                let available_refs: Vec<String> = snapshot
                    .elements
                    .iter()
                    .take(10)
                    .map(|e| {
                        format!(
                            "  {}  {} {}",
                            e.ref_id,
                            e.element_type,
                            e.label.as_deref().map(|l| format!("\"{}\"", l)).unwrap_or_default()
                        )
                    })
                    .collect();

                format!(
                    "Element not found: {}\n\nAvailable refs in current snapshot:\n{}\n\nHint: Run 'agent-mobile snapshot' to see the current UI state.",
                    ref_id,
                    available_refs.join("\n")
                )
            })?;
            Ok(to_resolved(elem))
        }
        ElementTarget::Text(text) => {
            let elem = find_by_text(snapshot, text).ok_or_else(|| {
                format!(
                    "Element with text '{}' not found in snapshot.\n\nHint: Run 'agent-mobile snapshot' to see available elements.",
                    text
                )
            })?;
            Ok(to_resolved(elem))
        }
        ElementTarget::Key(_) => {
            // Keys don't need resolution - they're handled separately
            Err("Cannot resolve key target to element".into())
        }
        ElementTarget::Position(_) => {
            // Positions don't need snapshot resolution - they're handled separately
            Err("Cannot resolve position target to element".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_target_parse_ref() {
        let target = ElementTarget::parse("@e1");
        assert!(matches!(target, ElementTarget::Ref(r) if r == "@e1"));

        let target = ElementTarget::parse("@e42");
        assert!(matches!(target, ElementTarget::Ref(r) if r == "@e42"));
    }

    #[test]
    fn test_element_target_parse_coords() {
        let target = ElementTarget::parse("100,200");
        assert!(matches!(target, ElementTarget::Coords(100.0, 200.0)));

        let target = ElementTarget::parse("100.5, 200.5");
        assert!(matches!(target, ElementTarget::Coords(100.5, 200.5)));
    }

    #[test]
    fn test_element_target_parse_key() {
        let target = ElementTarget::parse("home");
        assert!(matches!(target, ElementTarget::Key(k) if k == "home"));

        let target = ElementTarget::parse("ENTER");
        assert!(matches!(target, ElementTarget::Key(k) if k == "enter"));
    }

    #[test]
    fn test_element_target_parse_text() {
        let target = ElementTarget::parse("Login");
        assert!(matches!(target, ElementTarget::Text(t) if t == "Login"));

        let target = ElementTarget::parse("Submit Button");
        assert!(matches!(target, ElementTarget::Text(t) if t == "Submit Button"));
    }

    #[test]
    fn test_resolved_element_center() {
        let elem = ResolvedElement {
            ref_id: "@e1".to_string(),
            element_type: "Button".to_string(),
            label: Some("Login".to_string()),
            frame: Frame {
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 30.0,
            },
            enabled: true,
            value: None,
            traits: vec![],
            placeholder: None,
            depth: 0,
            is_interactive: true,
        };
        assert_eq!(elem.center(), (125.0, 215.0));
    }

    #[test]
    fn test_resolved_element_is_text_input() {
        let elem = ResolvedElement {
            ref_id: "@e1".to_string(),
            element_type: "TextField".to_string(),
            label: None,
            frame: Frame::zero(),
            enabled: true,
            value: None,
            traits: vec![],
            placeholder: None,
            depth: 0,
            is_interactive: true,
        };
        assert!(elem.is_text_input());

        let elem = ResolvedElement {
            ref_id: "@e1".to_string(),
            element_type: "Button".to_string(),
            label: None,
            frame: Frame::zero(),
            enabled: true,
            value: None,
            traits: vec![],
            placeholder: None,
            depth: 0,
            is_interactive: true,
        };
        assert!(!elem.is_text_input());
    }
}
