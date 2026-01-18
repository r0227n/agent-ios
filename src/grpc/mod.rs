//! gRPC client for iOS companion communication.
//!
//! This module re-exports from `platform::ios::grpc` for backward compatibility.
//! New code should use `crate::platform::ios::grpc` directly.

// Re-export everything from the new location
pub use crate::platform::ios::grpc::client;
pub use crate::platform::ios::grpc::{IdbClient, LaunchConfig, XctraceTarget};

// Re-export proto types for backward compatibility
pub use crate::platform::ios::proto::idb;
