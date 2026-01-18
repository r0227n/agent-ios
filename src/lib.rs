//! agent-mobile - Rust-based iOS Development Bridge
//!
//! This crate provides both a CLI tool and a library API for controlling
//! iOS devices and simulators.
//!
//! # Features
//!
//! - **CLI Tool**: Drop-in replacement for Python `idb` CLI
//! - **AI Commands**: Simplified commands for AI agent automation
//! - **Library API**: Programmatic access for Rust projects
//!
//! # Library Usage
//!
//! ```ignore
//! use agent_mobile::api::IosDevice;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to a device
//!     let mut device = IosDevice::connect(None).await?;
//!
//!     // Take a screenshot
//!     let screenshot = device.screenshot().await?;
//!     std::fs::write("screenshot.png", screenshot)?;
//!
//!     // Tap at coordinates
//!     device.tap(100.0, 200.0).await?;
//!
//!     // Type text
//!     device.type_text("Hello!").await?;
//!
//!     // Swipe
//!     device.swipe((100.0, 500.0), (100.0, 200.0), Some(0.5)).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! The crate is organized into several modules:
//!
//! - `api` - High-level programmatic API
//! - `platform::ios` - iOS-specific implementations (gRPC, companion, simctl)
//! - `core` - Shared types, errors, and traits
//! - `cli` - CLI command implementations

// Re-export modules for library use
pub mod api;
pub mod cli;
pub mod companion;
pub mod core;
pub mod grpc;
pub mod platform;
pub mod simctl;
pub mod types;

// Convenience re-exports
pub use api::IosDevice;
pub use grpc::{IdbClient, LaunchConfig};
pub use types::{Address, Compression, TargetDescription, TargetType};
