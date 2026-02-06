//! iOS platform implementation for agent-mobile.
//!
//! This crate provides iOS-specific functionality including:
//! - XCUITest Runner HTTP communication
//! - Companion daemon management (legacy)
//! - Simulator management via simctl
//! - HID event generation (legacy)

pub mod companion;
pub mod grpc;
pub mod hid;
pub mod proto;
pub mod simctl;
pub mod snapshot;
pub mod xcuitest;

// Re-export main types
pub use companion::{CompanionLister, CompanionResolver, CompanionState};
pub use grpc::{IdbClient, LaunchConfig};
pub use hid::events::{
    button_to_events, key_sequence_to_events, key_to_events, swipe_to_events, tap_to_events,
    text_to_events,
};
pub use simctl::{
    boot, clone, create, delete, delete_all, erase, get_booted_simulator, install_app,
    io_screenshot_bytes, list_apps, shutdown, uninstall_app, BootedSimulator, ImageFormat,
    SimctlAppInfo,
};
pub use snapshot::extract_ios_elements;
pub use xcuitest::{ensure_runner_started, XCUITestClient};
