//! Integration tests for CLI feature commands.
//!
//! This file aggregates all CLI feature integration tests from the cli/ subdirectory.
//! These tests verify the `agent-mobile <feature>` commands (excluding `idb` subcommand).

// Import idb common utilities
#[path = "idb/common/mod.rs"]
mod idb_common;

// CLI feature common utilities
#[path = "cli/common/mod.rs"]
mod common;

// Feature integration tests
#[path = "cli/app_integration.rs"]
mod app_integration;

#[path = "cli/device_integration.rs"]
mod device_integration;

#[path = "cli/gesture_integration.rs"]
mod gesture_integration;

#[path = "cli/keyboard_integration.rs"]
mod keyboard_integration;

#[path = "cli/navigator_integration.rs"]
mod navigator_integration;

#[path = "cli/clipboard_integration.rs"]
mod clipboard_integration;

#[path = "cli/privacy_integration.rs"]
mod privacy_integration;
