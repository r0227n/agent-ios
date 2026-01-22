//! Ref Resolution System
//!
//! @e1, @e2 形式の ref 識別子を座標に変換するシステム。

use std::path::Path;

use crate::cli::helpers::CommandResult;
use crate::cli::snapshot::types::{Frame, Snapshot, SnapshotElement};

/// Target type for Core Commands
#[derive(Debug, Clone)]
pub enum Target {
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
        }
    }
}

impl Target {
    /// Parse a target string into a Target type
    pub fn parse(s: &str) -> Self {
        let s = s.trim();

        // Check for @eN reference format
        if s.starts_with('@') {
            return Target::Ref(s.to_string());
        }

        // Check for coordinate format (x,y)
        if let Some((x, y)) = parse_coords(s) {
            return Target::Coords(x, y);
        }

        // Check for special positions (e.g., "center")
        if is_special_position(s) {
            return Target::Position(s.to_lowercase());
        }

        // Check for special keys
        if is_special_key(s) {
            return Target::Key(s.to_lowercase());
        }

        // Treat as text to search
        Target::Text(s.to_string())
    }

    /// Check if this target is a special key
    #[allow(dead_code)]
    pub fn is_key(&self) -> bool {
        matches!(self, Target::Key(_))
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

/// Load a snapshot from a JSON file
pub fn load_snapshot_from_file(path: &Path) -> CommandResult<Snapshot> {
    if !path.exists() {
        return Err(format!(
            "Snapshot file not found: {}\n\nHint: Run 'agent-mobile snapshot --format json -o <file>' to create a snapshot",
            path.display()
        )
        .into());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read snapshot file '{}': {}", path.display(), e))?;

    let snapshot: Snapshot = serde_json::from_str(&content).map_err(|e| {
        format!(
            "Failed to parse snapshot file '{}': {}\n\nHint: Make sure the file was created with '--format json'",
            path.display(),
            e
        )
    })?;

    Ok(snapshot)
}

/// Find an element by ref in a snapshot
pub fn find_by_ref<'a>(snapshot: &'a Snapshot, ref_id: &str) -> Option<&'a SnapshotElement> {
    snapshot.elements.iter().find(|e| e.ref_id == ref_id)
}

/// Find an element by text (label or value) in a snapshot
pub fn find_by_text<'a>(snapshot: &'a Snapshot, text: &str) -> Option<&'a SnapshotElement> {
    // First try exact match on label
    if let Some(elem) = snapshot
        .elements
        .iter()
        .find(|e| e.label.as_ref().map(|l| l == text).unwrap_or(false))
    {
        return Some(elem);
    }

    // Then try exact match on value
    if let Some(elem) = snapshot
        .elements
        .iter()
        .find(|e| e.value.as_ref().map(|v| v == text).unwrap_or(false))
    {
        return Some(elem);
    }

    // Then try case-insensitive match on label
    let text_lower = text.to_lowercase();
    if let Some(elem) = snapshot.elements.iter().find(|e| {
        e.label
            .as_ref()
            .map(|l| l.to_lowercase() == text_lower)
            .unwrap_or(false)
    }) {
        return Some(elem);
    }

    // Finally try partial match on label (contains)
    snapshot.elements.iter().find(|e| {
        e.label
            .as_ref()
            .map(|l| l.to_lowercase().contains(&text_lower))
            .unwrap_or(false)
    })
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
    }
}

/// Resolve a target to an element from a snapshot
pub fn resolve_from_snapshot(
    snapshot: &Snapshot,
    target: &Target,
) -> CommandResult<ResolvedElement> {
    match target {
        Target::Coords(x, y) => Ok(ResolvedElement::from_coords(*x, *y)),
        Target::Ref(ref_id) => {
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
        Target::Text(text) => {
            let elem = find_by_text(snapshot, text).ok_or_else(|| {
                format!(
                    "Element with text '{}' not found in snapshot.\n\nHint: Run 'agent-mobile snapshot' to see available elements.",
                    text
                )
            })?;
            Ok(to_resolved(elem))
        }
        Target::Key(_) => {
            // Keys don't need resolution - they're handled separately
            Err("Cannot resolve key target to element".into())
        }
        Target::Position(_) => {
            // Positions don't need snapshot resolution - they're handled separately
            Err("Cannot resolve position target to element".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_parse_ref() {
        let target = Target::parse("@e1");
        assert!(matches!(target, Target::Ref(r) if r == "@e1"));

        let target = Target::parse("@e42");
        assert!(matches!(target, Target::Ref(r) if r == "@e42"));
    }

    #[test]
    fn test_target_parse_coords() {
        let target = Target::parse("100,200");
        assert!(matches!(target, Target::Coords(100.0, 200.0)));

        let target = Target::parse("100.5, 200.5");
        assert!(matches!(target, Target::Coords(100.5, 200.5)));
    }

    #[test]
    fn test_target_parse_key() {
        let target = Target::parse("home");
        assert!(matches!(target, Target::Key(k) if k == "home"));

        let target = Target::parse("ENTER");
        assert!(matches!(target, Target::Key(k) if k == "enter"));
    }

    #[test]
    fn test_target_parse_text() {
        let target = Target::parse("Login");
        assert!(matches!(target, Target::Text(t) if t == "Login"));

        let target = Target::parse("Submit Button");
        assert!(matches!(target, Target::Text(t) if t == "Submit Button"));
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
        };
        assert!(!elem.is_text_input());
    }
}
