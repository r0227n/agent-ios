//! Android platform implementation for agent-mobile.
//!
//! This crate provides Android-specific functionality including:
//! - ADB integration for device communication
//! - UI Automator for accessibility and interaction
//! - Screenshot capture
//! - Permission management

pub mod adb;
pub mod snapshot;

// Re-export main types
pub use adb::app::AppInfo;
pub use adb::connection::AdbConnection;
pub use adb::uiautomator::{
    dump_ui, find_by_id, find_by_text, find_by_type, parse_ui_hierarchy, AccessibilityElement,
};
pub use adb::{
    get_android_version, get_api_level, get_avd_name, get_device_model, is_adb_available,
    is_emulator, list_avds, list_devices, AdbError, Result,
};
pub use snapshot::extract_android_elements;

// Re-export screenshot and permission functions
pub use adb::permission::{
    grant_permission, list_permissions, reset_permissions, revoke_permission,
};
pub use adb::screenshot::{screenshot, screenshot_bytes};
