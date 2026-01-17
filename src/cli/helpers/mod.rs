//! CLI helper modules for reducing boilerplate across commands
//!
//! This module provides common utilities for CLI command implementations:
//!
//! - [`client`]: Connection helpers (`with_client`, `CommandResult`)
//! - [`file_container`]: FileContainer building utilities
//! - [`signal`]: Ctrl+C signal handling
//! - [`output`]: Output destination (stdout/file) abstraction

pub mod client;
pub mod file_container;
pub mod output;
pub mod signal;

// Re-export commonly used items
pub use client::{with_client, CommandResult};
pub use file_container::{file_container, file_container_with_root, DefaultContainer};
pub use output::OutputWriter;
pub use signal::setup_ctrl_c_handler;
