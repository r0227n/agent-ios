# agent-mobile-gateway

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

High-level unified API gateway for agent-mobile, providing platform-agnostic device automation.

## Overview

`agent-mobile-gateway` is the top-level API layer in the agent-mobile workspace. It provides a clean, unified interface for mobile device operations that abstracts away platform-specific details from both iOS and Android implementations.

**Key Features**:
- **Platform Abstraction**: Single API for iOS and Android operations
- **Auto-Detection**: Automatic platform discovery based on connected devices
- **High-Level Operations**: Simplified device control (tap, swipe, launch, screenshot)
- **Unified Console Streaming**: Stream logs from any platform with consistent interface
- **Simplified Connection**: No need to manage gRPC clients or ADB commands directly

**Design Philosophy**:
- Hide complexity: Platform-specific code stays in `platform-ios` and `platform-android`
- Consistent API: Same method signatures across platforms where possible
- Sensible defaults: Auto-connect, auto-detect, minimal configuration
- Easy integration: Suitable for both CLI and library use

## Architecture

### Gateway Pattern

```
┌─────────────────────────────────────────────────────────┐
│                    agent-mobile CLI                     │
│                  (Command dispatcher)                   │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│              agent-mobile-gateway                       │
│           (Platform-agnostic API layer)                 │
├─────────────────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────┐  ┌──────────┐ │
│  │ DeviceResolver │  │   IosDevice    │  │ Android  │ │
│  │ (Auto-detect)  │  │  (High-level)  │  │  Device  │ │
│  └────────┬───────┘  └────────┬───────┘  └────┬─────┘ │
└───────────┼──────────────────┼────────────────┼───────┘
            │                  │                │
            ▼                  ▼                ▼
┌─────────────────┐  ┌──────────────────┐  ┌─────────────┐
│ platform-ios    │  │ platform-android │  │   console   │
│ (gRPC+simctl)   │  │   (ADB+XML)      │  │  streaming  │
└─────────────────┘  └──────────────────┘  └─────────────┘
```

### Layer Responsibilities

| Layer | Responsibility | Example |
|-------|----------------|---------|
| **CLI** | Parse arguments, dispatch commands | `agent-mobile tap @e1` |
| **Gateway** | Platform routing, unified API | `IosDevice::connect()` |
| **Platform** | Platform-specific implementation | `IdbClient::hid()` |

## Key Components

### `DeviceResolver` - Platform Detection

**Purpose**: Automatically detect which platform (iOS or Android) has available devices.

**Main Functions**:

```rust
pub struct DeviceResolver;

impl DeviceResolver {
    /// Parse platform from string ("ios" or "android")
    pub fn parse_platform(s: &str) -> Result<Platform>;

    /// Detect platform based on available devices
    pub async fn detect_platform() -> Result<Platform>;

    /// Check if iOS devices are available
    pub async fn has_ios_devices() -> bool;

    /// Check if Android devices are available
    pub async fn has_android_devices() -> bool;
}
```

**Detection Logic**:
- **iOS**: Checks `/tmp/idb/state` for companion daemons
- **Android**: Runs `adb devices -l` and parses output

**Example**:

```rust
use agent_mobile_gateway::DeviceResolver;
use agent_mobile_core::Platform;

// Auto-detect platform
let platform = DeviceResolver::detect_platform().await?;
match platform {
    Platform::Ios => println!("Using iOS"),
    Platform::Android => println!("Using Android"),
}

// Manual check
if DeviceResolver::has_ios_devices().await {
    println!("iOS devices available");
}
```

### `IosDevice` - High-Level iOS API

**Purpose**: Simplified, high-level API for iOS device operations.

**Main Methods**:

```rust
pub struct IosDevice {
    client: IdbClient,
    udid: String,
}

impl IosDevice {
    /// Connect to an iOS device (auto-select if udid is None)
    pub async fn connect(udid: Option<&str>) -> Result<Self>;

    /// Take a screenshot
    pub async fn screenshot(&self) -> Result<Vec<u8>>;

    /// Tap at coordinates
    pub async fn tap(&mut self, x: f64, y: f64) -> Result<()>;

    /// Long press
    pub async fn long_press(&mut self, x: f64, y: f64, duration: f64) -> Result<()>;

    /// Swipe gesture
    pub async fn swipe(
        &mut self,
        start: (f64, f64),
        end: (f64, f64),
        duration: Option<f64>
    ) -> Result<()>;

    /// Input text
    pub async fn input_text(&mut self, text: &str) -> Result<()>;

    /// Launch an app
    pub async fn launch_app(&mut self, bundle_id: &str) -> Result<()>;

    /// Terminate an app
    pub async fn terminate_app(&mut self, bundle_id: &str) -> Result<()>;

    /// Stream device logs
    pub async fn stream_logs(
        &mut self,
        source: LogSource,
        args: Vec<String>
    ) -> Result<Streaming<LogResponse>>;

    /// Get the device UDID
    pub fn udid(&self) -> &str;
}
```

**Internal Implementation**:
- Wraps `IdbClient` from `platform-ios`
- Manages companion connection via `CompanionResolver`
- Converts high-level calls to gRPC requests

**Example**:

```rust
use agent_mobile_gateway::IosDevice;

// Connect to any available iOS device
let mut device = IosDevice::connect(None).await?;

// Perform operations
device.tap(100.0, 200.0).await?;
device.swipe((100.0, 400.0), (100.0, 100.0), Some(0.3)).await?;
device.input_text("test@example.com").await?;

// Take screenshot
let png_data = device.screenshot().await?;
std::fs::write("screenshot.png", png_data)?;

// Launch app
device.launch_app("com.example.app").await?;
```

### `AndroidDevice` - High-Level Android API

**Purpose**: Simplified, high-level API for Android device operations.

**Main Methods**:

```rust
pub struct AndroidDevice {
    udid: Option<String>,
}

impl AndroidDevice {
    /// Connect to an Android device (auto-select if udid is None)
    pub async fn connect(udid: Option<&str>) -> Result<Self>;

    /// Stream device logs via adb logcat
    pub async fn stream_logs(&mut self) -> Result<LogcatStream>;

    /// Get the device UDID
    pub fn udid(&self) -> Option<&str>;
}

pub struct LogcatStream {
    child: Child,
    reader: tokio::io::Lines<BufReader<ChildStdout>>,
}

impl LogcatStream {
    /// Read the next line from logcat
    pub async fn next_line(&mut self) -> Result<Option<String>>;

    /// Stop the logcat stream
    pub async fn stop(&mut self) -> Result<()>;
}
```

**Internal Implementation**:
- Wraps ADB commands from `platform-android`
- Manages `adb logcat` process lifecycle
- Provides async stream interface for logs

**Example**:

```rust
use agent_mobile_gateway::AndroidDevice;

// Connect to any available Android device
let mut device = AndroidDevice::connect(None).await?;

// Stream logs
let mut stream = device.stream_logs().await?;

while let Some(line) = stream.next_line().await? {
    println!("LOG: {}", line);
}

stream.stop().await?;
```

### `stream_console_logs` - Unified Console Streaming

**Purpose**: Platform-agnostic console log streaming with graceful shutdown.

**Function Signature**:

```rust
pub async fn stream_console_logs(
    platform: Platform,
    udid: Option<&str>,
    writer: OutputWriter,
    stop_rx: watch::Receiver<bool>,
) -> Result<()>;
```

**Parameters**:
- `platform` - Target platform (iOS or Android)
- `udid` - Optional device identifier
- `writer` - Output destination (stdout or file)
- `stop_rx` - Signal for graceful shutdown (e.g., Ctrl+C)

**Behavior**:
- **iOS**: Uses gRPC streaming via `IosDevice::stream_logs`
- **Android**: Uses `adb logcat` via `AndroidDevice::stream_logs`
- **Shutdown**: Responds to `stop_rx` signal for clean termination

**Example**:

```rust
use agent_mobile_gateway::stream_console_logs;
use agent_mobile_core::{Platform, OutputWriter};
use tokio::sync::watch;

// Setup graceful shutdown
let (stop_tx, stop_rx) = watch::channel(false);

// Spawn log streaming
tokio::spawn(async move {
    stream_console_logs(
        Platform::Ios,
        None,
        OutputWriter::from_path("-").unwrap(),
        stop_rx,
    ).await
});

// On Ctrl+C signal
stop_tx.send(true)?;
```

## Platform Detection

The gateway automatically detects available platforms using environment clues:

### iOS Detection

Checks for running idb_companion daemons:

```rust
// Looks for: /tmp/idb/state
// Format: JSON array of companion info
// Example: [{"udid": "...", "address": {"domain_socket": {"path": "..."}}}]

pub async fn has_ios_devices() -> bool {
    let state_path = Path::new("/tmp/idb/state");
    if !state_path.exists() {
        return false;
    }
    // Parse JSON and check for non-empty array
}
```

### Android Detection

Queries ADB for connected devices:

```rust
// Runs: adb devices -l
// Output:
//   List of devices attached
//   emulator-5554     device product:...

pub async fn has_android_devices() -> bool {
    let output = Command::new("adb").args(["devices", "-l"]).output().await;
    // Parse output and check for "device" state
}
```

### Auto-Selection Priority

When both platforms have devices:
1. **iOS first** - Checked before Android
2. **Android fallback** - If iOS unavailable
3. **Error** - If neither available

## Usage Examples

### Platform-Agnostic Operations

```rust
use agent_mobile_gateway::{DeviceResolver, IosDevice, AndroidDevice};
use agent_mobile_core::Platform;

// Auto-detect and connect
let platform = DeviceResolver::detect_platform().await?;

match platform {
    Platform::Ios => {
        let mut device = IosDevice::connect(None).await?;
        device.tap(100.0, 200.0).await?;
        let screenshot = device.screenshot().await?;
    }
    Platform::Android => {
        let mut device = AndroidDevice::connect(None).await?;
        // Android-specific operations via platform-android directly
    }
}
```

### Unified Console Streaming

```rust
use agent_mobile_gateway::stream_console_logs;
use agent_mobile_core::{Platform, OutputWriter};
use tokio::sync::watch;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup shutdown signal
    let (stop_tx, stop_rx) = watch::channel(false);

    // Spawn Ctrl+C handler
    tokio::spawn(async move {
        signal::ctrl_c().await.ok();
        stop_tx.send(true).ok();
    });

    // Stream logs
    stream_console_logs(
        Platform::Ios,
        None,  // Auto-select device
        OutputWriter::from_path("-")?,  // stdout
        stop_rx,
    ).await?;

    Ok(())
}
```

### Library Integration

```rust
use agent_mobile_gateway::IosDevice;

// As a library, you can integrate device operations into your own code

pub async fn automate_login(email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut device = IosDevice::connect(None).await?;

    // Tap email field
    device.tap(200.0, 300.0).await?;
    device.input_text(email).await?;

    // Tap password field
    device.tap(200.0, 400.0).await?;
    device.input_text(password).await?;

    // Tap login button
    device.tap(200.0, 500.0).await?;

    // Wait for navigation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Take screenshot for verification
    let screenshot = device.screenshot().await?;
    std::fs::write("login_result.png", screenshot)?;

    Ok(())
}
```

### Explicit Platform Selection

```rust
use agent_mobile_gateway::{DeviceResolver, IosDevice};
use agent_mobile_core::Platform;

// Parse platform from string
let platform = DeviceResolver::parse_platform("ios")?;

// Connect based on platform
match platform {
    Platform::Ios => {
        let device = IosDevice::connect(Some("UDID-123")).await?;
        // iOS operations
    }
    Platform::Android => {
        // Android operations via platform-android
    }
}
```

## API Design Principles

### Consistent Interface

Where possible, the gateway provides consistent method signatures:

```rust
// Both platforms support connect
IosDevice::connect(udid: Option<&str>) -> Result<Self>
AndroidDevice::connect(udid: Option<&str>) -> Result<Self>

// Both support log streaming (different return types internally)
ios_device.stream_logs(...) -> Result<Streaming<LogResponse>>
android_device.stream_logs() -> Result<LogcatStream>
```

### Platform Differences

Some operations are platform-specific:

| Operation | iOS | Android | Gateway Approach |
|-----------|-----|---------|------------------|
| Screenshot | ✓ | △ | `IosDevice` only |
| App Launch | ✓ | ✓ | Both via `platform-*` |
| HID Input | ✓ | △ | `IosDevice` only |
| Log Streaming | ✓ | ✓ | Unified via `stream_console_logs` |

For operations not in the gateway, use platform crates directly:

```rust
// iOS-specific: Use platform-ios
use agent_mobile_platform_ios::simctl;
simctl::boot("UDID-123")?;

// Android-specific: Use platform-android
use agent_mobile_platform_android::adb::app;
app::launch(Some("emulator-5554"), "com.example.app").await?;
```

## Dependencies

**Platform crates** (implementation):
- `agent-mobile-core` - Common types and traits
- `agent-mobile-platform-ios` - iOS implementation
- `agent-mobile-platform-android` - Android implementation

**Runtime**:
- `tokio` 1.49 - Async runtime with process, fs, sync features
- `tokio-stream` 0.1 - Stream utilities
- `tonic` 0.12 - gRPC support (for iOS)

**Utilities**:
- `serde` + `serde_json` - Serialization
- `thiserror` + `anyhow` - Error handling

## License

See the [main repository LICENSE](../../LICENSE) for details.

## Related Documentation

- [Main README](../../README.md) - CLI usage and installation
- [agent-mobile-core](../core/README.md) - Platform-agnostic types and traits
- [agent-mobile-platform-ios](../platform-ios/README.md) - iOS implementation details
- [agent-mobile-platform-android](../platform-android/README.md) - Android implementation details
