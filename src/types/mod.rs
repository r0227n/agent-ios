//! Type definitions for agent-mobile.
//!
//! This module re-exports types from the core module for backward compatibility.
//! New code should use `crate::core::types` directly.

// Keep the old modules for tests and backward compatibility
pub mod install;
pub mod merge;
pub mod output;
pub mod target;

// Re-export from the old modules (for backward compatibility with existing code)
pub use install::{Compression, InstalledArtifact};
pub use merge::merge_connected_targets;
pub use output::{human_format_target, json_format_target};
pub use target::{Address, CompanionInfo, TargetDescription, TargetType};
