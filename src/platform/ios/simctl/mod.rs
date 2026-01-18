//! Direct xcrun simctl integration.
//!
//! This module provides direct access to iOS Simulator management
//! through the `xcrun simctl` command line tool.

pub mod management;

pub use management::{boot, clone, create, delete, delete_all, erase, shutdown};
