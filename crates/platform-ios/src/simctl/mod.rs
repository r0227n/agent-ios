//! Direct xcrun simctl integration.
//!
//! This module provides direct access to iOS Simulator management
//! through the `xcrun simctl` command line tool.

pub mod management;

pub use management::{
    boot, clone, create, delete, delete_all, erase, get_booted_simulator, install_app,
    io_screenshot_bytes, list_apps, list_simulators, shutdown, uninstall_app, BootedSimulator,
    ImageFormat, SimctlAppInfo,
};
