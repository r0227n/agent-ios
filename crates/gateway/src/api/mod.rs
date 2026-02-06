//! Programmatic API for mobile automation.
//!
//! This module provides a high-level API for controlling Android devices,
//! designed for use as a library in other Rust projects.

pub mod android;

pub use android::AndroidDevice;
