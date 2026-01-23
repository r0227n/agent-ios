//! Core types and traits for agent-mobile.
//!
//! This crate provides platform-agnostic types, error definitions, and traits
//! that are used throughout the agent-mobile workspace.

pub mod error;
pub mod snapshot;
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use error::{Error, Result};
pub use snapshot::{extract_traits_for_type, is_interactive_type, Frame, RawElement};
pub use types::{
    human_format_target, json_format_target, merge_connected_targets, Address, CompanionInfo,
    Compression, InstalledArtifact, Platform, ScrollDirection, TargetDescription, TargetType,
};
