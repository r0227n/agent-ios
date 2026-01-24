//! ADB (Android Debug Bridge) command interface.
//!
//! This module provides functions for communicating with Android devices
//! and emulators via ADB.

pub mod app;
mod commands;
pub mod input;
pub mod permission;
pub mod screenshot;
pub mod uiautomator;

pub use commands::*;
