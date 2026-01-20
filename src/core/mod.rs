//! Core module containing shared types, error definitions, and platform traits.
//!
//! This module provides platform-agnostic types and abstractions that are used
//! throughout the agent-mobile codebase.

pub mod error;
pub mod traits;
pub mod types;

// Re-export commonly used types (allow unused for now as migration progresses)
#[allow(unused_imports)]
pub use error::{Error, Result};
#[allow(unused_imports)]
pub use types::{
    human_format_target, json_format_target, merge_connected_targets, Address, CompanionInfo,
    Compression, InstalledArtifact, Platform, ScrollDirection, TargetDescription, TargetType,
};
