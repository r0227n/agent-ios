//! Integration tests for CLI feature commands.
//!
//! This file aggregates all CLI feature integration tests from the cli/ subdirectory.
//! These tests verify the `agent-mobile <feature>` commands.

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

#[path = "cli/clipboard_integration.rs"]
mod clipboard_integration;

#[path = "cli/privacy_integration.rs"]
mod privacy_integration;

// Phase 1: Core element tests
#[path = "cli/find_integration.rs"]
mod find_integration;

#[path = "cli/get_integration.rs"]
mod get_integration;

#[path = "cli/is_integration.rs"]
mod is_integration;

// Phase 2: UI operation tests
#[path = "cli/snapshot_integration.rs"]
mod snapshot_integration;

#[path = "cli/wait_integration.rs"]
mod wait_integration;

#[path = "cli/check_integration.rs"]
mod check_integration;

#[path = "cli/fill_integration.rs"]
mod fill_integration;

#[path = "cli/select_integration.rs"]
mod select_integration;

// Phase 3: Screen capture tests
#[path = "cli/screenshot_integration.rs"]
mod screenshot_integration;

#[path = "cli/record_integration.rs"]
mod record_integration;

#[path = "cli/console_integration.rs"]
mod console_integration;

// Phase 4: Session management tests
#[path = "cli/session_integration.rs"]
mod session_integration;
