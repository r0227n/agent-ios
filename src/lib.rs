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
//! The crate is organized as a workspace with the following structure:
//!
//! - `agent-mobile-core` - Shared types and traits
//! - `agent-mobile-platform-ios` - iOS-specific implementations
//! - `agent-mobile-platform-android` - Android-specific implementations
//! - `agent-mobile-gateway` - High-level API
//!
//! For backward compatibility, this crate re-exports the workspace crates.

// CLI module (local implementation)
pub mod cli;

// Re-export workspace crates directly
pub use agent_mobile_core;
pub use agent_mobile_gateway;
pub use agent_mobile_platform_android;
pub use agent_mobile_platform_ios;

// Convenience re-exports from workspace crates
pub use agent_mobile_core::{
    Address, Compression, Platform, ScrollDirection, TargetDescription, TargetType,
};
pub use agent_mobile_gateway::IosDevice;
pub use agent_mobile_platform_ios::{IdbClient, LaunchConfig};
