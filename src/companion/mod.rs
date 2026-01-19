//! Companion daemon management.
//!
//! This module re-exports from `platform::ios::companion` for backward compatibility.
//! New code should use `crate::platform::ios::companion` directly.

// Re-export main types
pub use crate::platform::ios::companion::{CompanionLister, CompanionResolver, CompanionState};
