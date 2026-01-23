//! CLI helper modules for reducing boilerplate across commands
//!
//! This module provides common utilities for CLI command implementations:
//!
//! - [`client`]: Connection helpers (`with_client`, `CommandResult`)
//! - [`common_args`]: Shared argument structs (`DeviceArgs`, `DeviceFormatArgs`)
//! - [`file_container`]: FileContainer building utilities
//! - [`format`]: Output format options (`OutputFormat`)
//! - [`signal`]: Ctrl+C signal handling
//! - [`output`]: Output destination (stdout/file) abstraction
//! - [`time`]: Time format parsing utilities

pub mod client;
pub mod common_args;
pub mod file_container;
pub mod format;
pub mod signal;
pub mod time;

// Re-export commonly used items
pub use client::{with_client, with_client_streaming, CommandResult};
pub use common_args::{DeviceArgs, DeviceFormatArgs};
pub use file_container::{file_container, file_container_with_root, DefaultContainer};
pub use format::OutputFormat;
pub use signal::setup_ctrl_c_handler;
pub use time::formatted_time_to_seconds;

// Re-export from core
pub use agent_mobile_core::OutputWriter;
