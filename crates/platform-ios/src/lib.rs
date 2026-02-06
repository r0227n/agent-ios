//! iOS platform implementation for agent-mobile.
//!
//! This crate provides iOS-specific functionality including:
//! - XCUITest Runner HTTP communication
//! - Simulator management via simctl

pub mod simctl;
pub mod snapshot;
pub mod xcuitest;

// Re-export main types
pub use simctl::{
    boot, clone, create, delete, delete_all, erase, get_booted_simulator, install_app,
    io_screenshot_bytes, list_apps, list_simulators, shutdown, uninstall_app, BootedSimulator,
    ImageFormat, SimctlAppInfo,
};
pub use snapshot::extract_ios_elements;
pub use xcuitest::{ensure_runner_started, XCUITestClient};
