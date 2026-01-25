//! gRPC client implementation for iOS companion communication.
//!
//! This module provides the IdbClient which communicates with idb_companion
//! via gRPC over Unix domain sockets or TCP.

pub mod app;
pub mod client;
pub mod device;
pub mod hid;
pub mod target;

pub use client::{IdbClient, LaunchConfig};
