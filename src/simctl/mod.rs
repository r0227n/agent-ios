//! Direct xcrun simctl integration.
//!
//! This module re-exports from `platform::ios::simctl` for backward compatibility.
//! New code should use `crate::platform::ios::simctl` directly.

// Re-export functions
pub use crate::platform::ios::simctl::{boot, clone, create, delete, delete_all, erase, shutdown};
