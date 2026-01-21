//! Reference ID generation for snapshot elements.
//!
//! Assigns `@e1`, `@e2`, etc. to elements, prioritizing interactive elements.

use super::types::{RawElement, SnapshotElement};

/// Convert a tree of RawElements into a flat list of SnapshotElements with refs.
///
/// Interactive elements get refs assigned first (@e1, @e2, ...),
/// then remaining elements get refs in tree order.
pub fn generate_refs(raw_elements: &[RawElement]) -> Vec<SnapshotElement> {
    // First pass: flatten the tree and collect all elements
    let mut flat_elements: Vec<FlatElement> = Vec::new();
    for raw in raw_elements {
        flatten_tree(raw, 0, None, &mut flat_elements);
    }

    // Build parent-child relationships
    let mut elements: Vec<SnapshotElement> = flat_elements
        .iter()
        .map(|fe| SnapshotElement {
            ref_id: String::new(), // Will be assigned later
            element_type: fe.raw.element_type.clone(),
            label: fe.raw.label.clone(),
            frame: fe.raw.frame.clone(),
            enabled: fe.raw.enabled,
            traits: fe.raw.traits.clone(),
            placeholder: fe.raw.placeholder.clone(),
            value: fe.raw.value.clone(),
            children_indices: Vec::new(),
            depth: fe.depth,
            is_interactive: fe.raw.is_interactive(),
            parent_index: fe.parent_index,
        })
        .collect();

    // Set children_indices based on parent_index
    for i in 0..elements.len() {
        if let Some(parent_idx) = elements[i].parent_index {
            elements[parent_idx].children_indices.push(i);
        }
    }

    // Collect interactive and non-interactive indices
    let interactive_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(_, e)| e.is_interactive)
        .map(|(i, _)| i)
        .collect();

    let non_interactive_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(_, e)| !e.is_interactive)
        .map(|(i, _)| i)
        .collect();

    // Assign refs: interactive elements first, then non-interactive
    let mut ref_counter = 1;

    for idx in interactive_indices {
        elements[idx].ref_id = format!("@e{}", ref_counter);
        ref_counter += 1;
    }

    for idx in non_interactive_indices {
        elements[idx].ref_id = format!("@e{}", ref_counter);
        ref_counter += 1;
    }

    elements
}

/// Temporary structure for flattening.
struct FlatElement<'a> {
    raw: &'a RawElement,
    depth: u32,
    parent_index: Option<usize>,
}

/// Flatten a tree of RawElements into a list while preserving depth info.
fn flatten_tree<'a>(
    element: &'a RawElement,
    depth: u32,
    parent_index: Option<usize>,
    result: &mut Vec<FlatElement<'a>>,
) {
    let current_index = result.len();
    result.push(FlatElement {
        raw: element,
        depth,
        parent_index,
    });

    for child in &element.children {
        flatten_tree(child, depth + 1, Some(current_index), result);
    }
}

/// Check if an element at the given index has interactive descendants.
pub fn has_interactive_descendants(elements: &[SnapshotElement], index: usize) -> bool {
    let element = &elements[index];

    if element.is_interactive {
        return true;
    }

    for &child_idx in &element.children_indices {
        if has_interactive_descendants(elements, child_idx) {
            return true;
        }
    }

    false
}

/// Check if an element is "empty" (has no label and no value).
pub fn is_empty_structure(element: &SnapshotElement) -> bool {
    element.label.is_none() && element.value.is_none() && !element.is_interactive
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::snapshot::types::Frame;

    fn make_raw(element_type: &str, label: Option<&str>, children: Vec<RawElement>) -> RawElement {
        RawElement {
            element_type: element_type.to_string(),
            label: label.map(String::from),
            frame: Frame::zero(),
            enabled: true,
            traits: Vec::new(),
            placeholder: None,
            value: None,
            children,
        }
    }

    #[test]
    fn test_generate_refs_basic() {
        let raw = vec![
            make_raw("Button", Some("Login"), vec![]),
            make_raw("StaticText", Some("Welcome"), vec![]),
        ];

        let result = generate_refs(&raw);

        assert_eq!(result.len(), 2);
        // Button is interactive, gets @e1
        assert_eq!(result[0].ref_id, "@e1");
        assert!(result[0].is_interactive);
        // StaticText is not interactive, gets @e2
        assert_eq!(result[1].ref_id, "@e2");
        assert!(!result[1].is_interactive);
    }

    #[test]
    fn test_generate_refs_interactive_first() {
        let raw = vec![make_raw(
            "Window",
            Some("Main"),
            vec![
                make_raw("StaticText", Some("Title"), vec![]),
                make_raw("Button", Some("Submit"), vec![]),
                make_raw("TextField", Some("Email"), vec![]),
            ],
        )];

        let result = generate_refs(&raw);

        assert_eq!(result.len(), 4);

        // Find elements by type
        let button = result.iter().find(|e| e.element_type == "Button").unwrap();
        let text_field = result
            .iter()
            .find(|e| e.element_type == "TextField")
            .unwrap();
        let static_text = result
            .iter()
            .find(|e| e.element_type == "StaticText")
            .unwrap();
        let window = result.iter().find(|e| e.element_type == "Window").unwrap();

        // Interactive elements should have lower ref numbers
        let button_num: u32 = button.ref_id[2..].parse().unwrap();
        let text_field_num: u32 = text_field.ref_id[2..].parse().unwrap();
        let static_text_num: u32 = static_text.ref_id[2..].parse().unwrap();
        let window_num: u32 = window.ref_id[2..].parse().unwrap();

        assert!(button_num < static_text_num);
        assert!(text_field_num < window_num);
    }

    #[test]
    fn test_depth_tracking() {
        let raw = vec![make_raw(
            "Window",
            Some("Main"),
            vec![make_raw(
                "View",
                None,
                vec![make_raw("Button", Some("Deep"), vec![])],
            )],
        )];

        let result = generate_refs(&raw);

        let window = result.iter().find(|e| e.element_type == "Window").unwrap();
        let view = result.iter().find(|e| e.element_type == "View").unwrap();
        let button = result.iter().find(|e| e.element_type == "Button").unwrap();

        assert_eq!(window.depth, 0);
        assert_eq!(view.depth, 1);
        assert_eq!(button.depth, 2);
    }

    #[test]
    fn test_children_indices() {
        let raw = vec![make_raw(
            "Window",
            Some("Main"),
            vec![
                make_raw("Button", Some("A"), vec![]),
                make_raw("Button", Some("B"), vec![]),
            ],
        )];

        let result = generate_refs(&raw);
        let window_idx = result
            .iter()
            .position(|e| e.element_type == "Window")
            .unwrap();
        let window = &result[window_idx];

        assert_eq!(window.children_indices.len(), 2);
    }

    #[test]
    fn test_has_interactive_descendants() {
        let raw = vec![make_raw(
            "View",
            None,
            vec![make_raw(
                "View",
                None,
                vec![make_raw("Button", Some("Deep"), vec![])],
            )],
        )];

        let result = generate_refs(&raw);
        let top_view_idx = result.iter().position(|e| e.depth == 0).unwrap();

        assert!(has_interactive_descendants(&result, top_view_idx));
    }

    #[test]
    fn test_is_empty_structure() {
        let element = SnapshotElement {
            ref_id: "@e1".to_string(),
            element_type: "View".to_string(),
            label: None,
            frame: Frame::zero(),
            enabled: true,
            traits: Vec::new(),
            placeholder: None,
            value: None,
            children_indices: Vec::new(),
            depth: 0,
            is_interactive: false,
            parent_index: None,
        };

        assert!(is_empty_structure(&element));

        let labeled_element = SnapshotElement {
            label: Some("Title".to_string()),
            ..element.clone()
        };
        assert!(!is_empty_structure(&labeled_element));
    }
}
