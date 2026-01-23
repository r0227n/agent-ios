//! Data types for UI snapshot representation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use agent_mobile_core::snapshot::Frame;

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
