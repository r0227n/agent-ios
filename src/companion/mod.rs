//! Companion daemon management.
//!
//! This module re-exports from `platform::ios::companion` for backward compatibility.
//! New code should use `crate::platform::ios::companion` directly.

// Re-export submodules from the new location
pub use crate::platform::ios::companion::lister;
pub use crate::platform::ios::companion::resolver;
pub use crate::platform::ios::companion::spawner;
pub use crate::platform::ios::companion::state;

// Re-export main types
pub use crate::platform::ios::companion::{CompanionLister, CompanionResolver, CompanionState};
