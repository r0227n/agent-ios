//! Programmatic API for mobile automation.
//!
//! This module provides a high-level API for controlling iOS and Android devices,
//! designed for use as a library in other Rust projects.
//!
//! # Example
//!
//! ```ignore
//! use agent_mobile_gateway::IosDevice;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to a device (auto-selects if only one available)
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
//!     device.type_text("Hello, World!").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod android;
pub mod ios;

pub use android::AndroidDevice;
pub use ios::IosDevice;
