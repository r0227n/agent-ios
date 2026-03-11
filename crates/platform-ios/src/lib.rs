//! iOS platform implementation for agent-mobile.
//!
//! This crate provides iOS-specific functionality including:
//! - XCUITest Runner HTTP communication
//! - Simulator management via simctl

#[cfg(target_os = "macos")]
/// CoreSimulator-backed helpers for booted simulator app management.
pub mod coresim;
pub mod simctl;
pub mod snapshot;
/// XCUITest Runner client types and lifecycle helpers.
pub mod xcuitest;

// Re-export main types
pub use simctl::{
    boot, clone, create, delete, delete_all, erase, get_booted_simulator, install_app, list_apps,
    list_simulators, shutdown, uninstall_app, BootedSimulator, SimctlAppInfo,
};
pub use snapshot::extract_ios_elements;
pub use xcuitest::{ensure_runner_started, XCUITestClient};
