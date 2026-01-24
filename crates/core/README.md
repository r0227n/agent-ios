# agent-mobile-core

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

Platform-agnostic types, traits, and error handling for the agent-mobile workspace.

## Overview

`agent-mobile-core` provides the foundational abstractions used across all agent-mobile crates. It defines common types, traits for platform operations, error handling, and utilities that enable platform-independent code throughout the workspace.

This crate is designed to be a lightweight foundation that can be used by both platform implementations (iOS, Android) and higher-level APIs (gateway) without introducing circular dependencies.

**Key Design Principles**:
- **Platform Agnostic**: No platform-specific logic or dependencies
- **Trait-Based Abstraction**: Define interfaces that platform crates implement
- **Structured Errors**: Unified error type with context for debugging
- **Optional Features**: Keep dependencies minimal with feature flags

## Features

The crate provides two optional features:

- **`cli`** - Enables clap integration for CLI argument parsing (adds `clap` derive support)
- **`tonic`** - Enables gRPC error conversions (adds `tonic::Status` and `tonic::transport::Error` conversions)

Default configuration has no features enabled.

```toml
[dependencies]
agent-mobile-core = { path = "../core" }

# With CLI support
agent-mobile-core = { path = "../core", features = ["cli"] }

# With gRPC support
agent-mobile-core = { path = "../core", features = ["tonic"] }
```

## Public API

### Error Handling

The crate provides a unified error type with contextual variants:

```rust
use agent_mobile_core::{Error, Result};

pub enum Error {
    Grpc(String),              // gRPC communication errors
    Connection(String),         // Connection failures
    CompanionNotFound(String),  // Companion daemon not found
    TargetNotFound(String),     // Device/simulator not found
    MultipleTargets(String),    // Ambiguous target selection
    FileOperation(String),      // File system errors
    Installation(String),       // App installation errors
    InvalidArgument(String),    // Invalid parameters
    Io(std::io::Error),        // IO errors
    Other(String),             // Generic errors
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Error conversions** are provided for common types:
- `std::io::Error` → `Error::Io`
- `String` / `&str` → `Error::Other`
- `tonic::Status` → `Error::Grpc` (with `tonic` feature)
- `tonic::transport::Error` → `Error::Connection` (with `tonic` feature)

**Example usage**:

```rust
use agent_mobile_core::Result;

fn connect_to_device(udid: &str) -> Result<DeviceHandle> {
    if udid.is_empty() {
        return Err("UDID cannot be empty".into());
    }
    // ... connection logic
    Ok(DeviceHandle::new(udid))
}
```

### Output Formatting

`OutputWriter` provides a unified abstraction for writing command output to stdout or files:

```rust
use agent_mobile_core::OutputWriter;
use std::io::Write;

// Write to stdout
let mut writer = OutputWriter::from_path("-")?;

// Write to file
let mut writer = OutputWriter::from_path("output.png")?;

// Tee mode: write to both file and stdout
let mut writer = OutputWriter::tee_from_path("output.png")?;

writer.write_all(&data)?;
writer.flush()?;
```

### Trait System

The crate defines platform-agnostic operation traits that platform implementations must satisfy:

#### `DeviceOperations`

Core device interaction operations:

```rust
#[async_trait]
pub trait DeviceOperations: Send + Sync {
    /// Take a screenshot and return the image data
    async fn screenshot(&mut self) -> Result<Vec<u8>>;

    /// Tap at the specified coordinates
    async fn tap(&mut self, point: Point) -> Result<()>;

    /// Swipe from one point to another
    async fn swipe(&mut self, from: Point, to: Point, duration: Option<f64>) -> Result<()>;

    /// Input text
    async fn input_text(&mut self, text: &str) -> Result<()>;
}
```

#### `AppOperations`

Application lifecycle management:

```rust
#[async_trait]
pub trait AppOperations: Send + Sync {
    /// Install an application from the specified path
    async fn install(&mut self, path: &str) -> Result<String>;

    /// Launch an application by its bundle ID
    async fn launch(&mut self, bundle_id: &str) -> Result<u64>;

    /// Terminate a running application
    async fn terminate(&mut self, bundle_id: &str) -> Result<()>;

    /// Uninstall an application
    async fn uninstall(&mut self, bundle_id: &str) -> Result<()>;

    /// List installed applications
    async fn list_apps(&mut self) -> Result<Vec<AppInfo>>;
}
```

#### `FileOperations`

File system interactions:

```rust
#[async_trait]
pub trait FileOperations: Send + Sync {
    /// List files in a directory
    async fn list_files(&mut self, path: &str) -> Result<Vec<FileInfo>>;

    /// Pull a file from the device
    async fn pull(&mut self, remote_path: &str, local_path: &str) -> Result<()>;

    /// Push a file to the device
    async fn push(&mut self, local_path: &str, remote_path: &str) -> Result<()>;

    /// Remove a file or directory
    async fn remove(&mut self, path: &str) -> Result<()>;

    /// Create a directory
    async fn mkdir(&mut self, path: &str) -> Result<()>;
}
```

#### `MobileDevice`

Combined trait for full device capabilities:

```rust
pub trait MobileDevice: DeviceOperations + AppOperations + FileOperations {}

// Automatic blanket implementation
impl<T> MobileDevice for T where T: DeviceOperations + AppOperations + FileOperations {}
```

Any type implementing all three operation traits automatically implements `MobileDevice`.

### Types Module

#### Platform

Cross-platform device identification:

```rust
use agent_mobile_core::Platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Ios,
    Android,
}

// String conversions
let platform: Platform = "ios".parse()?;
assert_eq!(platform.as_str(), "ios");

// Display formatting
println!("Platform: {}", Platform::Ios); // "Platform: ios"
```

#### ScrollDirection

Gesture direction for scrolling and swiping:

```rust
use agent_mobile_core::ScrollDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

// Calculate swipe offsets for gesture execution
let (start, end) = ScrollDirection::Up.to_swipe_offsets(200.0);
// Returns: ((0.0, 100.0), (0.0, -100.0)) - swipe from bottom to top
```

#### Device Information

Target device identification and metadata:

```rust
pub struct DeviceInfo {
    pub udid: String,
    pub name: String,
    pub platform: Platform,
    pub arch: Option<String>,
    pub os_version: Option<String>,
    pub state: Option<String>,
}

pub enum TargetType {
    Simulator,
    Device,
}

pub struct Address {
    pub host: String,
    pub port: u16,
}

pub struct CompanionInfo {
    pub address: Address,
    pub udid: String,
    pub is_local: bool,
}
```

#### Installation Artifacts

App installation metadata:

```rust
pub struct InstalledArtifact {
    pub name: String,
    pub uuid: String,
}

pub enum Compression {
    GZIP,
    ZSTD,
}
```

### Snapshot Module

UI element extraction and representation:

#### Frame

Element position and size:

```rust
use agent_mobile_core::Frame;

pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

let frame = Frame { x: 100.0, y: 200.0, width: 50.0, height: 30.0 };
let (cx, cy) = frame.center(); // (125.0, 215.0)
```

#### RawElement

Intermediate representation of UI elements during extraction:

```rust
use agent_mobile_core::RawElement;

pub struct RawElement {
    pub element_type: String,
    pub label: Option<String>,
    pub frame: Frame,
    pub enabled: bool,
    pub traits: Vec<String>,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub children: Vec<RawElement>,
}

// Check if element is interactive
let button = RawElement { element_type: "Button".into(), /* ... */ };
assert!(button.is_interactive());
```

#### Interactive Element Detection

```rust
use agent_mobile_core::is_interactive_type;

// iOS types
assert!(is_interactive_type("Button"));
assert!(is_interactive_type("TextField"));
assert!(is_interactive_type("Link"));

// Android types
assert!(is_interactive_type("EditText"));
assert!(is_interactive_type("ImageButton"));
assert!(is_interactive_type("CheckBox"));

// Non-interactive types
assert!(!is_interactive_type("StaticText"));
assert!(!is_interactive_type("Image"));
```

## Usage Examples

### Implementing Platform Traits

```rust
use agent_mobile_core::{DeviceOperations, Point, Result};
use async_trait::async_trait;

struct MyDevice {
    udid: String,
}

#[async_trait]
impl DeviceOperations for MyDevice {
    async fn screenshot(&mut self) -> Result<Vec<u8>> {
        // Platform-specific screenshot implementation
        Ok(vec![])
    }

    async fn tap(&mut self, point: Point) -> Result<()> {
        println!("Tapping at ({}, {})", point.x, point.y);
        // Platform-specific tap implementation
        Ok(())
    }

    async fn swipe(&mut self, from: Point, to: Point, duration: Option<f64>) -> Result<()> {
        // Platform-specific swipe implementation
        Ok(())
    }

    async fn input_text(&mut self, text: &str) -> Result<()> {
        // Platform-specific text input implementation
        Ok(())
    }
}
```

### Using OutputWriter

```rust
use agent_mobile_core::OutputWriter;
use std::io::Write;

fn save_screenshot(data: &[u8], dest: &str) -> agent_mobile_core::Result<()> {
    let mut writer = OutputWriter::from_path(dest)?;
    writer.write_all(data)?;
    writer.flush()?;
    Ok(())
}

// Usage
save_screenshot(&image_data, "screenshot.png")?;  // Save to file
save_screenshot(&image_data, "-")?;               // Output to stdout
```

### Working with Platform Enum

```rust
use agent_mobile_core::Platform;

fn get_device_platform(os: &str) -> agent_mobile_core::Result<Platform> {
    os.parse().map_err(|e: String| e.into())
}

match get_device_platform("ios")? {
    Platform::Ios => println!("Using iOS implementation"),
    Platform::Android => println!("Using Android implementation"),
}
```

### Error Context

```rust
use agent_mobile_core::{Error, Result};

fn connect_with_context(host: &str, port: u16) -> Result<Connection> {
    connect(host, port).map_err(|e| {
        Error::Connection(format!("Failed to connect to {}:{}: {}", host, port, e))
    })
}
```

## Dependencies

**Core dependencies** (always included):
- `serde` 1.0 with derive - Serialization/deserialization
- `serde_json` 1.0 - JSON support
- `thiserror` 2.0 - Error derive macros
- `async-trait` 0.1 - Async trait support

**Optional dependencies**:
- `tonic` 0.12 (feature: `tonic`) - gRPC error conversions
- `clap` 4.5 with derive (feature: `cli`) - CLI argument parsing

**Dev dependencies**:
- `tempfile` 3.0 - Temporary file utilities for tests

## Architecture Integration

```
┌─────────────────────────────────────────────────────────┐
│                    agent-mobile CLI                     │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│               agent-mobile-gateway                      │
│         (High-level unified API layer)                  │
└─────────────────────────────────────────────────────────┘
                            │
          ┌─────────────────┴─────────────────┐
          ▼                                   ▼
┌─────────────────────┐           ┌─────────────────────┐
│ platform-ios        │           │ platform-android    │
│ (iOS gRPC+simctl)   │           │ (Android ADB)       │
└─────────────────────┘           └─────────────────────┘
          │                                   │
          └─────────────────┬─────────────────┘
                            ▼
                ┌───────────────────────┐
                │   agent-mobile-core   │ ← You are here
                │ (Types, Traits, Error)│
                └───────────────────────┘
```

**agent-mobile-core** serves as the foundation for the entire workspace:

- **Platform crates** (`platform-ios`, `platform-android`) implement the core traits
- **Gateway crate** uses core types and traits for the unified API
- **CLI** uses core types for argument parsing and output formatting

## License

See the [main repository LICENSE](../../LICENSE) for details.

## Related Documentation

- [Main README](../../README.md) - CLI usage and installation
- [agent-mobile-gateway](../gateway/README.md) - High-level unified API
- [agent-mobile-platform-ios](../platform-ios/README.md) - iOS implementation
- [agent-mobile-platform-android](../platform-android/README.md) - Android implementation
