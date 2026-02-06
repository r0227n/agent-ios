//! High-level API gateway for agent-mobile.
//!
//! This crate provides a unified, high-level API for mobile device automation,
//! abstracting over platform-specific implementations.

pub mod api;
pub mod console;
pub mod platform;

// Re-export main types
pub use api::android::AndroidDevice;
pub use console::stream_console_logs;
pub use platform::DeviceResolver;
