# agent-mobile-platform-ios

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

iOS platform implementation for agent-mobile, providing device and simulator automation via idb_companion gRPC and xcrun simctl.

## Overview

`agent-mobile-platform-ios` implements iOS-specific functionality for the agent-mobile workspace. It uses a **hybrid architecture** that combines:

- **idb_companion gRPC** - For runtime operations, streaming, HID input, and real device support
- **xcrun simctl** - For simulator lifecycle management and operations not available in the gRPC API

This approach provides the best of both worlds: the type safety and advanced features of gRPC where available, with fallback to Apple's official simctl for operations like simulator boot/shutdown.

**Key Features**:
- Type-safe gRPC communication with idb_companion daemon
- Unix domain socket support for high-performance local connections
- Automatic companion daemon discovery and connection
- Simulator lifecycle management (boot, shutdown, create, delete)
- HID event generation for touch and keyboard input
- Accessibility tree extraction for UI automation
- Protocol Buffer definitions from Facebook's IDB project

## Architecture

### Hybrid Approach: gRPC + simctl

This crate employs a **hybrid implementation strategy** where operations are routed to either gRPC or simctl based on capability and efficiency:

```
┌─────────────────────────────────────────────────────────┐
│               platform-ios (This Crate)                 │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────┐         ┌─────────────────┐      │
│  │   gRPC Client   │         │  simctl Wrapper │      │
│  │  (IdbClient)    │         │  (Command exec) │      │
│  └────────┬────────┘         └────────┬────────┘      │
│           │                           │                │
│           ▼                           ▼                │
│  ┌─────────────────┐         ┌─────────────────┐      │
│  │ Unix Socket     │         │ xcrun simctl    │      │
│  │ /tmp/...        │         │ boot/shutdown   │      │
│  └────────┬────────┘         └─────────────────┘      │
└───────────┼──────────────────────────────────────────┘
            │
            ▼
   ┌────────────────┐
   │ idb_companion  │ (Swift/ObjC daemon)
   └────────┬───────┘
            │
            ▼
   ┌────────────────┐
   │ iOS Device or  │
   │   Simulator    │
   └────────────────┘
```

### When to Use gRPC vs simctl

#### Use gRPC (`IdbClient`) When:

| Capability | Reason |
|------------|--------|
| **Streaming Required** | Install progress, log streaming |
| **HID Input** | Tap, swipe, text input - simctl cannot do this |
| **Real Device Support** | Must work with physical devices |
| **Type Safety** | Structured errors and responses |
| **Advanced Features** | Accessibility info, framebuffer access |

**Examples**: `screenshot`, `hid_event`, `install_app`, `launch_app`, `accessibility_info`, `log_stream`

#### Use simctl (`xcrun simctl`) When:

| Capability | Reason |
|------------|--------|
| **Lifecycle Management** | No `boot`/`shutdown` RPC in proto |
| **Simulator Creation** | Only simctl can create/delete simulators |
| **No Proto API** | Feature not exposed via gRPC |
| **Fallback** | When gRPC encounters errors |

**Examples**: `boot`, `shutdown`, `create`, `delete`, `erase`, `clone`, `pbcopy`, `pbpaste`

#### Decision Matrix

| Feature | gRPC | simctl | Choice | Reason |
|---------|------|--------|--------|--------|
| Screenshot | ✓ | ✓ | **gRPC** | Framebuffer direct access |
| App install | ✓ | ✓ | **gRPC** | Progress streaming |
| App launch | ✓ | ✓ | **gRPC** | Better error handling |
| HID input | ✓ | ✗ | **gRPC** | Only option |
| Accessibility | ✓ | ✗ | **gRPC** | Only option |
| Boot simulator | △ | ✓ | **simctl** | No RPC in proto |
| Create simulator | ✗ | ✓ | **simctl** | gRPC unsupported |
| Clipboard | ✗ | ✓ | **simctl** | gRPC unsupported |

See [`.claude/rules/cli-feature.md`](../../.claude/rules/cli-feature.md) for detailed decision guidelines.

## Key Modules

### `grpc` - gRPC Client and Operations

**Purpose**: Type-safe communication with idb_companion daemon via Unix domain sockets or TCP.

**Main Types**:

```rust
pub struct IdbClient {
    client: CompanionServiceClient<Channel>,
    address: Address,
}

pub struct LaunchConfig {
    pub bundle_id: String,
    pub app_args: Vec<String>,
    pub env: HashMap<String, String>,
    pub foreground_if_running: bool,
    pub wait_for_debugger: bool,
}

pub enum XctraceTarget {
    AllProcesses,
    Attach(String),
    Launch { process: String, args: Vec<String>, /* ... */ },
}
```

**Key Features**:
- Unix domain socket support for high-performance local connections
- Automatic retry with exponential backoff
- Connection pooling and timeout management
- Streaming support for install progress and logs

**Sub-modules**:
- `grpc/client.rs` - Core gRPC client implementation
- `grpc/app.rs` - Application lifecycle (install, launch, terminate)
- `grpc/device.rs` - Device operations (accessibility, describe, clear_keychain)
- `grpc/file.rs` - File system operations (list, pull, push, mkdir, rm)
- `grpc/hid.rs` - HID event streaming
- `grpc/media.rs` - Media operations (photos, videos)
- `grpc/target.rs` - Target discovery and connection
- `grpc/test.rs` - XCTest operations
- `grpc/debug.rs` - Debugging operations (DAP, instruments)

**Example**:

```rust
use agent_mobile_platform_ios::grpc::IdbClient;

// Connect via Unix domain socket
let client = IdbClient::connect_uds("/tmp/idb_companion.sock").await?;

// Take a screenshot
let png_data = client.screenshot().await?;

// Launch an app
let launch_config = LaunchConfig {
    bundle_id: "com.example.app".into(),
    app_args: vec![],
    env: HashMap::new(),
    foreground_if_running: true,
    wait_for_debugger: false,
};
let pid = client.launch(launch_config).await?;
```

### `companion` - Daemon Discovery and Management

**Purpose**: Locate, spawn, and manage idb_companion daemon processes.

**Main Types**:

```rust
pub struct CompanionResolver {
    state: CompanionState,
    spawner: Option<CompanionSpawner>,
}

pub struct CompanionLister {
    state: CompanionState,
}

pub enum CompanionState {
    // Tracks running companions
}

pub struct CompanionSpawnConfig {
    pub notify_fd: Option<i32>,
    pub device_set_path: Option<String>,
}
```

**Key Features**:
- **Auto-discovery**: Finds running companions via state file (`~/.idb-companion-state.json`)
- **Auto-spawning**: Launches companion daemon when needed
- **UDID Resolution**: Priority-based selection (session > explicit > auto-detection)
- **Connection Management**: Establishes gRPC connections

**UDID Resolution Priority**:
1. Session UDID (highest) - from `session create`
2. Explicit UDID - from `--udid` flag
3. Auto-detection (lowest) - if only one companion exists

**Sub-modules**:
- `companion/resolver.rs` - Companion resolution and auto-connection
- `companion/lister.rs` - List available companions
- `companion/spawner.rs` - Spawn new companion processes
- `companion/state.rs` - Persistent state management

**Example**:

```rust
use agent_mobile_platform_ios::companion::{CompanionResolver, CompanionLister};

// List all available companions
let lister = CompanionLister::new()?;
let companions = lister.list_available().await?;

// Resolve and connect to a companion
let resolver = CompanionResolver::new(None)?; // None = no auto-spawn
let client = resolver.resolve_and_connect(Some("UDID-123")).await?;
```

### `simctl` - Simulator Lifecycle Management

**Purpose**: Direct wrapper around `xcrun simctl` for simulator operations not available via gRPC.

**Main Functions**:

```rust
pub fn boot(udid: &str) -> Result<()>;
pub fn shutdown(udid: &str) -> Result<()>;
pub fn create(name: &str, device_type: &str, runtime: &str) -> Result<String>;
pub fn delete(udid: &str) -> Result<()>;
pub fn delete_all() -> Result<()>;
pub fn erase(udid: &str) -> Result<()>;
pub fn clone(udid: &str, new_name: &str) -> Result<String>;
pub fn io_screenshot_bytes(udid: &str, format: ImageFormat) -> Result<Vec<u8>>;
```

**Image Formats**:

```rust
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Gif,
    Tiff,
}
```

**Error Handling**:
- Idempotent operations: `boot` on booted simulator is OK
- Clear error messages from simctl stderr
- Automatic state detection

**Example**:

```rust
use agent_mobile_platform_ios::simctl::{boot, shutdown, create, erase, ImageFormat};

// Boot a simulator
boot("UDID-123")?;

// Create a new simulator
let new_udid = create(
    "Test iPhone",
    "iPhone 15 Pro",
    "iOS 17.0"
)?;

// Take a screenshot (alternative to gRPC)
let png_bytes = io_screenshot_bytes("UDID-123", ImageFormat::Png)?;

// Reset simulator to clean state
erase("UDID-123")?;

// Shutdown
shutdown("UDID-123")?;
```

### `hid` - HID Event Generation

**Purpose**: Convert high-level gestures (tap, swipe, text) into low-level HID events for the gRPC HIDEvent RPC.

**Main Functions**:

```rust
pub fn tap_to_events(x: f64, y: f64) -> Vec<proto::HidEvent>;
pub fn swipe_to_events(x1: f64, y1: f64, x2: f64, y2: f64, duration: f64) -> Vec<proto::HidEvent>;
pub fn text_to_events(text: &str) -> Vec<proto::HidEvent>;
pub fn key_to_events(keycode: u32) -> Vec<proto::HidEvent>;
pub fn button_to_events(button: proto::HidButtonType) -> Vec<proto::HidEvent>;
```

**Event Types**:
- `Touch` - Finger touch down/move/up
- `Key` - Keyboard key press/release
- `Button` - Hardware button (home, lock, volume)

**Example**:

```rust
use agent_mobile_platform_ios::hid::events::{tap_to_events, swipe_to_events, text_to_events};

// Generate tap event
let tap_events = tap_to_events(100.0, 200.0);
client.hid_event(tap_events).await?;

// Generate swipe event (200ms duration)
let swipe_events = swipe_to_events(100.0, 400.0, 100.0, 100.0, 0.2);
client.hid_event(swipe_events).await?;

// Generate text input events
let text_events = text_to_events("Hello, World!");
client.hid_event(text_events).await?;
```

### `snapshot` - Accessibility Tree Extraction

**Purpose**: Extract structured UI element data from iOS accessibility hierarchy.

**Main Function**:

```rust
pub fn extract_ios_elements(
    raw_snapshot: &proto::AccessibilityInfoResponse
) -> Vec<agent_mobile_core::RawElement>;
```

**Process**:
1. Receive accessibility info from gRPC (`describe` RPC)
2. Parse protobuf accessibility elements
3. Extract frame, label, type, traits, value
4. Flatten hierarchy into list of interactive elements
5. Convert to `agent_mobile_core::RawElement` for snapshot command

**Example**:

```rust
use agent_mobile_platform_ios::snapshot::extract_ios_elements;

// Get accessibility info via gRPC
let accessibility_info = client.describe_accessibility().await?;

// Extract elements
let elements = extract_ios_elements(&accessibility_info);

// Filter interactive elements
let interactive: Vec<_> = elements.iter()
    .filter(|e| e.is_interactive())
    .collect();
```

### `proto` - Generated Protocol Buffer Code

**Purpose**: Auto-generated Rust types from `proto/idb.proto`.

**Build Process**:
- `build.rs` uses `tonic-build` to compile `.proto` files
- Generates service clients and message types
- Output: `proto/idb.rs` (included via `include!` macro)

**Key Generated Types**:
- `CompanionServiceClient` - Main gRPC service client
- `InstallRequest`, `LaunchRequest`, `ScreenshotRequest`, etc.
- `AccessibilityInfoResponse`, `InstalledAppInfo`, etc.

**Example**:

```rust
use crate::proto::idb::{
    companion_service_client::CompanionServiceClient,
    InstallRequest, Payload, FileContainer,
};

// Proto types are used internally by IdbClient
let request = InstallRequest {
    payload: Some(Payload {
        file_path: Some(app_path.into()),
        // ...
    }),
    // ...
};
```

## Usage Examples

### Connecting to idb_companion

```rust
use agent_mobile_platform_ios::grpc::IdbClient;

// Option 1: Connect via Unix domain socket (preferred for local use)
let socket_path = "/tmp/idb_companion_UDID.sock";
let client = IdbClient::connect_uds(socket_path).await?;

// Option 2: Connect via TCP (for remote devices)
use agent_mobile_core::types::Address;
let address = Address { host: "localhost".into(), port: 10882 };
let client = IdbClient::connect_tcp(&address).await?;

// Option 3: Auto-resolve using companion resolver
use agent_mobile_platform_ios::companion::CompanionResolver;
let resolver = CompanionResolver::new(None)?;
let client = resolver.resolve_and_connect(Some("UDID-123")).await?;
```

### Managing Simulators

```rust
use agent_mobile_platform_ios::simctl::{boot, shutdown, create, delete, erase};

// Create a new simulator
let udid = create(
    "Test Device",
    "iPhone 15 Pro",
    "iOS 17.0"
)?;

// Boot the simulator
boot(&udid)?;

// ... use the simulator ...

// Reset to clean state
erase(&udid)?;

// Shutdown
shutdown(&udid)?;

// Delete when done
delete(&udid)?;
```

### Sending HID Events

```rust
use agent_mobile_platform_ios::{IdbClient, hid::events::*};

let mut client = IdbClient::connect_uds(socket_path).await?;

// Tap at coordinates
let events = tap_to_events(100.0, 200.0);
client.hid_event(events).await?;

// Swipe (e.g., scroll up)
let events = swipe_to_events(200.0, 500.0, 200.0, 200.0, 0.3);
client.hid_event(events).await?;

// Type text
let events = text_to_events("test@example.com");
client.hid_event(events).await?;

// Press hardware button
let events = button_to_events(proto::HidButtonType::Home);
client.hid_event(events).await?;
```

### Taking Screenshots

```rust
use agent_mobile_platform_ios::{IdbClient, simctl};

// Option 1: via gRPC (preferred - works on real devices)
let mut client = IdbClient::connect_uds(socket_path).await?;
let png_data = client.screenshot().await?;
std::fs::write("screenshot.png", png_data)?;

// Option 2: via simctl (simulators only)
let png_data = simctl::io_screenshot_bytes("UDID", simctl::ImageFormat::Png)?;
std::fs::write("screenshot.png", png_data)?;
```

### Extracting UI Elements

```rust
use agent_mobile_platform_ios::{IdbClient, snapshot::extract_ios_elements};

let mut client = IdbClient::connect_uds(socket_path).await?;

// Get accessibility info
let accessibility_info = client.describe_accessibility().await?;

// Extract elements
let elements = extract_ios_elements(&accessibility_info);

// Find button by label
let login_button = elements.iter()
    .find(|e| e.element_type == "Button" && e.label.as_deref() == Some("Login"));

if let Some(button) = login_button {
    let (cx, cy) = button.frame.center();
    println!("Login button at ({}, {})", cx, cy);
}
```

## External Dependencies

### idb_companion

**What it is**: Swift/Objective-C daemon from Facebook's IDB project that provides device communication.

**Installation**:
```bash
# Via Homebrew
brew tap facebook/fb
brew install idb-companion

# Or build from source
git clone https://github.com/facebook/idb.git
cd idb
./idb_build.sh idb_companion build
```

**Usage**:
- Auto-spawned by `CompanionResolver` when needed
- Manually: `idb_companion --udid <UDID> --port 10882`
- Creates Unix socket at `/tmp/idb_<UDID>_companion.sock`

### Xcode Command Line Tools

**Required for**:
- `xcrun simctl` commands (simulator management)
- Simulator runtime availability

**Installation**:
```bash
xcode-select --install
```

**Verification**:
```bash
xcrun simctl list devices
```

## Build System

### Protocol Buffer Compilation

This crate uses `tonic-build` in `build.rs` to compile protobuf definitions:

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)  // Client-only
        .compile(&["proto/idb.proto"], &["proto"])?;
    Ok(())
}
```

**Generated code**: `target/<profile>/build/agent-mobile-platform-ios-*/out/idb.rs`

**Inclusion**: Via `include!` macro in `src/proto/mod.rs`

### Building the Crate

```bash
# Build (triggers proto compilation)
cargo build --release

# Test
cargo test

# Check proto changes
git diff proto/idb.proto
```

## Dependencies

**Core**:
- `agent-mobile-core` - Platform-agnostic types and traits
- `tonic` 0.12 - gRPC client framework
- `prost` 0.13 - Protocol Buffer runtime
- `tokio` 1.49 - Async runtime with process, net, signal features

**gRPC Transport**:
- `tower` 0.4 - Service middleware
- `hyper-util` 0.1 - Unix socket support

**Utilities**:
- `serde` + `serde_json` - Serialization
- `thiserror` + `anyhow` - Error handling
- `async-trait` - Async trait support
- `regex` - Pattern matching
- `nix` 0.29 - Unix signal handling
- `tempfile` - Temporary file creation
- `flate2` + `tar` - Archive extraction (for trace files)

**Build**:
- `tonic-build` 0.12 - Proto compilation

## License

See the [main repository LICENSE](../../LICENSE) for details.

## Related Documentation

- [Main README](../../README.md) - CLI usage and installation
- [agent-mobile-core](../core/README.md) - Platform-agnostic types and traits
- [IDB Architecture](../../src/idb/README.md) - IDB command implementation details
- [cli-feature.md](../../.claude/rules/cli-feature.md) - gRPC vs simctl decision guide
- [Facebook IDB](https://github.com/facebook/idb) - Original Python implementation
- [proto/idb.proto](../../proto/idb.proto) - Protocol Buffer definitions
