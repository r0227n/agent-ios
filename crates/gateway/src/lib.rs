//! High-level API gateway for agent-mobile.
//!
//! This crate provides a unified, high-level API for mobile device automation,
//! abstracting over platform-specific implementations.

pub mod api;

// Re-export main types
pub use api::ios::IosDevice;
