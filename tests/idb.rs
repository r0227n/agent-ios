// Integration tests for IDB commands
// This file aggregates all IDB integration tests from the idb/ subdirectory

#[path = "idb/common/mod.rs"]
mod common;

#[path = "idb/accessibility_integration.rs"]
mod accessibility_integration;

#[path = "idb/contacts_keychain_integration.rs"]
mod contacts_keychain_integration;

#[path = "idb/crash_integration.rs"]
mod crash_integration;

#[path = "idb/dap_integration.rs"]
mod dap_integration;

#[path = "idb/debugserver_integration.rs"]
mod debugserver_integration;

#[path = "idb/dsym_integration.rs"]
mod dsym_integration;

#[path = "idb/dylib_integration.rs"]
mod dylib_integration;

#[path = "idb/file_ls_integration.rs"]
mod file_ls_integration;

#[path = "idb/file_mv_integration.rs"]
mod file_mv_integration;

#[path = "idb/file_pull_integration.rs"]
mod file_pull_integration;

#[path = "idb/file_push_integration.rs"]
mod file_push_integration;

#[path = "idb/file_read_write_integration.rs"]
mod file_read_write_integration;

#[path = "idb/file_rm_integration.rs"]
mod file_rm_integration;

#[path = "idb/file_tail_integration.rs"]
mod file_tail_integration;

#[path = "idb/focus_integration.rs"]
mod focus_integration;

#[path = "idb/framework_integration.rs"]
mod framework_integration;

#[path = "idb/hid_integration.rs"]
mod hid_integration;

#[path = "idb/install_integration.rs"]
mod install_integration;

#[path = "idb/instruments_integration.rs"]
mod instruments_integration;

#[path = "idb/kill_integration.rs"]
mod kill_integration;

#[path = "idb/launch_integration.rs"]
mod launch_integration;

#[path = "idb/list_apps_integration.rs"]
mod list_apps_integration;

#[path = "idb/list_targets_integration.rs"]
mod list_targets_integration;

#[path = "idb/location_integration.rs"]
mod location_integration;

#[path = "idb/log_integration.rs"]
mod log_integration;

#[path = "idb/media_add_integration.rs"]
mod media_add_integration;

#[path = "idb/memory_warning_integration.rs"]
mod memory_warning_integration;

#[path = "idb/mkdir_integration.rs"]
mod mkdir_integration;

#[path = "idb/notification_integration.rs"]
mod notification_integration;

#[path = "idb/permissions_integration.rs"]
mod permissions_integration;

#[path = "idb/photos_clear_integration.rs"]
mod photos_clear_integration;

#[path = "idb/screenshot_integration.rs"]
mod screenshot_integration;

#[path = "idb/settings_integration.rs"]
mod settings_integration;

#[path = "idb/shell_integration.rs"]
mod shell_integration;

#[path = "idb/target_boot_shutdown_integration.rs"]
mod target_boot_shutdown_integration;

#[path = "idb/target_connection_integration.rs"]
mod target_connection_integration;

#[path = "idb/target_create_delete_integration.rs"]
mod target_create_delete_integration;

#[path = "idb/terminate_integration.rs"]
mod terminate_integration;

#[path = "idb/uninstall_integration.rs"]
mod uninstall_integration;

#[path = "idb/url_integration.rs"]
mod url_integration;

#[path = "idb/video_integration.rs"]
mod video_integration;

#[path = "idb/xctest_install_integration.rs"]
mod xctest_install_integration;

#[path = "idb/xctest_integration.rs"]
mod xctest_integration;

#[path = "idb/xctrace_integration.rs"]
mod xctrace_integration;
