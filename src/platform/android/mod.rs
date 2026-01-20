//! Android platform support module.
//!
//! This module provides functionality for communicating with Android devices
//! and emulators via ADB (Android Debug Bridge).

pub mod adb;
mod emulator_list;

pub use emulator_list::EmulatorLister;
