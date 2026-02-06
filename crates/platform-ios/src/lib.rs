//! iOS platform implementation for agent-mobile.
//!
//! This crate provides iOS-specific functionality including:
//! - XCUITest Runner HTTP communication
//! - Simulator management via simctl

<<<<<<< HEAD
pub mod coresim;
||||||| parent of 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
=======
#[cfg(target_os = "macos")]
pub mod coresim;
>>>>>>> 07dcdae9 (fix: Android screenshot - file + pull 方式に変更して JPEG 対応完了)
pub mod simctl;
pub mod snapshot;
pub mod xcuitest;

// Re-export main types
pub use simctl::{
    boot, clone, create, delete, delete_all, erase, get_booted_simulator, install_app, list_apps,
    list_simulators, shutdown, uninstall_app, BootedSimulator, SimctlAppInfo,
};
pub use snapshot::extract_ios_elements;
pub use xcuitest::{ensure_runner_started, XCUITestClient};
