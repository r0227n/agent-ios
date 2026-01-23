//! Tree-format text output for snapshots.

use super::ref_generator::{has_interactive_descendants, is_empty_structure};
use super::types::SnapshotElement;

/// Options for tree printing.
#[derive(Debug, Clone, Default)]
pub struct PrintOptions {
    /// Only show interactive elements (and their parents with interactive descendants)
    pub interactive_only: bool,

    /// Remove empty structural elements
    pub compact: bool,

    /// Maximum depth to display (None = unlimited)
    pub max_depth: Option<u32>,
}

/// Print elements in tree format to a String.
pub fn print_tree(elements: &[SnapshotElement], options: &PrintOptions) -> String {
    let mut output = String::new();

    // Find root elements (depth = 0)
    let root_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(_, e)| e.depth == 0)
        .map(|(i, _)| i)
        .collect();

    for root_idx in root_indices {
        print_element(&mut output, elements, root_idx, "", true, options);
    }

    output
}

/// Print a single element and its children.
fn print_element(
    output: &mut String,
    elements: &[SnapshotElement],
    index: usize,
    prefix: &str,
    is_last: bool,
    options: &PrintOptions,
) {
    let element = &elements[index];

    // Check depth limit
    if let Some(max_depth) = options.max_depth {
        if element.depth > max_depth {
            return;
        }
    }

    // Check if should be displayed
    if !should_display(elements, index, options) {
        return;
    }

    // Build the current line
    let connector = if element.depth == 0 {
        ""
    } else if is_last {
        "\u{2514}\u{2500}\u{2500} " // "└── "
    } else {
        "\u{251c}\u{2500}\u{2500} " // "├── "
    };

    let line = format_element_line(element);
    output.push_str(&format!("{}{}{}\n", prefix, connector, line));

    // Get children that should be displayed
    let children: Vec<usize> = element
        .children_indices
        .iter()
        .copied()
        .filter(|&idx| should_display(elements, idx, options))
        .collect();

    // Calculate new prefix for children
    let child_prefix = if element.depth == 0 {
        String::new()
    } else if is_last {
        format!("{}    ", prefix) // 4 spaces
    } else {
        format!("{}\u{2502}   ", prefix) // "│   "
    };

    // Print children
    for (i, &child_idx) in children.iter().enumerate() {
        let is_last_child = i == children.len() - 1;
        print_element(
            output,
            elements,
            child_idx,
            &child_prefix,
            is_last_child,
            options,
        );
    }
}

/// Check if an element should be displayed based on options.
fn should_display(elements: &[SnapshotElement], index: usize, options: &PrintOptions) -> bool {
    let element = &elements[index];

    // Check depth limit
    if let Some(max_depth) = options.max_depth {
        if element.depth > max_depth {
            return false;
        }
    }

    // Interactive only filter
    if options.interactive_only
        && !element.is_interactive
        && !has_interactive_descendants(elements, index)
    {
        return false;
    }

    // Compact filter (remove empty structural elements)
    if options.compact && is_empty_structure(element) && element.children_indices.is_empty() {
        return false;
    }

    true
}

/// Format a single element line.
fn format_element_line(element: &SnapshotElement) -> String {
    let mut parts = Vec::new();

    // Element type
    parts.push(element.element_type.clone());

    // Label (quoted)
    if let Some(label) = &element.label {
        parts.push(format!("\"{}\"", truncate_string(label, 50)));
    }

    // Build base string
    let mut line = parts.join(" ");

    // Add ref
    line.push_str(&format!(" [ref={}]", element.ref_id));

    // Add placeholder if present
    if let Some(placeholder) = &element.placeholder {
        line.push_str(&format!(
            " placeholder=\"{}\"",
            truncate_string(placeholder, 30)
        ));
    }

    // Add value if present and different from label
    if let Some(value) = &element.value {
        if element.label.as_ref() != Some(value) {
            line.push_str(&format!(" value=\"{}\"", truncate_string(value, 30)));
        }
    }

    // Add disabled indicator
    if !element.enabled {
        line.push_str(" [disabled]");
    }

    line
}

/// Truncate a string with ellipsis if too long (handles UTF-8 properly).
fn truncate_string(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::types::Frame;

    fn make_element(
        ref_id: &str,
        element_type: &str,
        label: Option<&str>,
        depth: u32,
        is_interactive: bool,
        children_indices: Vec<usize>,
        parent_index: Option<usize>,
    ) -> SnapshotElement {
        SnapshotElement {
            ref_id: ref_id.to_string(),
            element_type: element_type.to_string(),
            label: label.map(String::from),
            frame: Frame::zero(),
            enabled: true,
            traits: Vec::new(),
            placeholder: None,
            value: None,
            children_indices,
            depth,
            is_interactive,
            parent_index,
        }
    }

    #[test]
    fn test_format_element_line() {
        let element = make_element("@e1", "Button", Some("Login"), 0, true, vec![], None);
        let line = format_element_line(&element);
        assert!(line.contains("Button"));
        assert!(line.contains("\"Login\""));
        assert!(line.contains("[ref=@e1]"));
    }

    #[test]
    fn test_format_element_line_with_placeholder() {
        let mut element = make_element("@e1", "TextField", Some("Email"), 0, true, vec![], None);
        element.placeholder = Some("Enter email".to_string());
        let line = format_element_line(&element);
        assert!(line.contains("placeholder=\"Enter email\""));
    }

    #[test]
    fn test_format_element_line_disabled() {
        let mut element = make_element("@e1", "Button", Some("Submit"), 0, true, vec![], None);
        element.enabled = false;
        let line = format_element_line(&element);
        assert!(line.contains("[disabled]"));
    }

    #[test]
    fn test_print_tree_simple() {
        let elements = vec![make_element(
            "@e1",
            "Button",
            Some("Login"),
            0,
            true,
            vec![],
            None,
        )];

        let output = print_tree(&elements, &PrintOptions::default());
        assert!(output.contains("Button \"Login\" [ref=@e1]"));
    }

    #[test]
    fn test_print_tree_nested() {
        let elements = vec![
            make_element("@e3", "Window", Some("Main"), 0, false, vec![1, 2], None),
            make_element("@e1", "Button", Some("A"), 1, true, vec![], Some(0)),
            make_element("@e2", "Button", Some("B"), 1, true, vec![], Some(0)),
        ];

        let output = print_tree(&elements, &PrintOptions::default());

        assert!(output.contains("Window"));
        assert!(output.contains("Button \"A\""));
        assert!(output.contains("Button \"B\""));
        // Check tree characters are present
        assert!(
            output.contains("\u{251c}\u{2500}\u{2500}")
                || output.contains("\u{2514}\u{2500}\u{2500}")
        );
    }

    #[test]
    fn test_print_tree_interactive_only() {
        let elements = vec![
            make_element("@e2", "View", None, 0, false, vec![1], None),
            make_element("@e1", "Button", Some("Submit"), 1, true, vec![], Some(0)),
        ];

        let options = PrintOptions {
            interactive_only: true,
            ..Default::default()
        };

        let output = print_tree(&elements, &options);

        // View should still be shown because it has interactive descendant
        assert!(output.contains("View"));
        assert!(output.contains("Button"));
    }

    #[test]
    fn test_print_tree_depth_limit() {
        let elements = vec![
            make_element("@e1", "Window", Some("Main"), 0, false, vec![1], None),
            make_element("@e2", "View", None, 1, false, vec![2], Some(0)),
            make_element("@e3", "Button", Some("Deep"), 2, true, vec![], Some(1)),
        ];

        let options = PrintOptions {
            max_depth: Some(1),
            ..Default::default()
        };

        let output = print_tree(&elements, &options);

        assert!(output.contains("Window"));
        assert!(output.contains("View"));
        assert!(!output.contains("Deep")); // Depth 2, should be hidden
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
    }

    #[test]
    fn test_truncate_string_utf8() {
        // Japanese characters should be handled correctly
        assert_eq!(truncate_string("ウィジェット", 10), "ウィジェット"); // 6 chars, no truncation
        assert_eq!(
            truncate_string("ウィジェット, スタック", 10), // 11 chars -> 7 + "..."
            "ウィジェット,..."
        );
        // Mixed ASCII and Japanese
        assert_eq!(truncate_string("Hello世界", 10), "Hello世界"); // 7 chars, no truncation
        assert_eq!(
            truncate_string("これは長いテキストです", 8),
            "これは長い..."
        ); // 11 chars -> 5 + "..."
    }
}
