//! CLI helper modules for reducing boilerplate across commands
//!
//! This module provides common utilities for CLI command implementations:
//!
//! - [`client`]: Connection helpers (`with_xcuitest`, `CommandResult`)
//! - [`common_args`]: Shared argument structs (`DeviceArgs`, `DeviceFormatArgs`)
//! - [`format`]: Output format options (`OutputFormat`)
//! - [`ios`]: Shared iOS helpers for screen metrics and device-specific logic
//! - [`signal`]: Ctrl+C signal handling
//! - [`output`]: Output destination (stdout/file) abstraction

pub mod client;
pub mod common_args;
pub mod format;
pub mod ios;
pub mod signal;
