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
