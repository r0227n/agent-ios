//! iOS platform implementation for agent-mobile.
//!
//! This crate provides iOS-specific functionality including:
//! - gRPC communication with idb_companion
//! - Companion daemon management
//! - Simulator management via simctl
//! - HID event generation

pub mod companion;
pub mod grpc;
pub mod hid;
pub mod proto;
pub mod simctl;
pub mod snapshot;

// Re-export main types
pub use companion::{CompanionLister, CompanionResolver, CompanionState};
pub use grpc::{IdbClient, LaunchConfig, XctraceTarget};
pub use hid::events::{
    button_to_events, key_sequence_to_events, key_to_events, swipe_to_events, tap_to_events,
    text_to_events,
};
pub use simctl::{
    boot, clone, create, delete, delete_all, erase, io_screenshot_bytes, shutdown, ImageFormat,
};
pub use snapshot::extract_ios_elements;
