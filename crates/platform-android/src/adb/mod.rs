//! ADB (Android Debug Bridge) command interface.
//!
//! This module provides functions for communicating with Android devices
//! and emulators via the native ADB protocol (TCP :5037).

pub mod app;
pub mod commands;
pub mod connection;
pub mod input;
pub mod logcat;
pub mod permission;
pub mod screenshot;
pub mod uiautomator;

pub use commands::*;
