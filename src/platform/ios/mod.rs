//! iOS platform implementation.
//!
//! This module provides iOS-specific functionality including:
//! - gRPC communication with idb_companion
//! - Companion daemon management
//! - Simulator management via simctl

pub mod companion;
pub mod grpc;
pub mod proto;
pub mod simctl;
