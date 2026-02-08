//! agent-mobile - AI-optimized CLI for iOS/Android device automation
//!
//! This crate is primarily a **CLI tool** for controlling iOS/Android devices
//! and simulators. It is designed to be invoked from the command line by AI
//! agents or developers.
//!
//! # Features
//!
//! - **CLI Tool**: AI-optimized CLI for iOS/Android device automation
//! - **AI Commands**: Simplified commands for AI agent automation
//!
//! # CLI Usage
//!
//! ```text
//! agent-mobile screenshot -o screenshot.png
//! agent-mobile tap 100 200
//! agent-mobile input "Hello!"
//! agent-mobile swipe up
//! ```
//!
//! For programmatic access from Rust, use the workspace crates directly:
//! - `agent_mobile_core` for shared types and traits
//! - `agent_mobile_platform_ios` for iOS operations
//! - `agent_mobile_platform_android` for Android operations
//! - `agent_mobile_gateway` for high-level API
//!
//! # Architecture
//!
//! The crate is organized as a workspace with the following structure:
//!
//! - `agent-mobile-core` - Shared types and traits
//! - `agent-mobile-platform-ios` - iOS-specific implementations
//! - `agent-mobile-platform-android` - Android-specific implementations
//! - `agent-mobile-gateway` - High-level API

// CLI modules (local implementation)
pub mod app;
pub mod command;
pub mod console;
pub mod core;
pub mod device;
pub mod doctor;
pub mod helpers;
pub mod record;
pub mod session;
pub mod snapshot;
