# agent-mobile-platform-ios

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

iOS platform implementation for agent-mobile, providing device and simulator automation via XCUITest Runner (HTTP) and xcrun simctl.

## Overview

`agent-mobile-platform-ios` implements iOS-specific functionality for the agent-mobile workspace. It uses a **hybrid architecture** that combines:

- **XCUITest Runner (HTTP)** - For runtime operations: touch, input, accessibility, app lifecycle
- **xcrun simctl** - For simulator lifecycle management: boot, shutdown, install, uninstall, screenshot

**Key Features**:
- HTTP/JSON communication with XCUITest Runner (localhost:8200)
- Simulator lifecycle management (boot, shutdown, install, uninstall)
- Device discovery via `simctl list` with caching (TTL 5s)
- Accessibility tree extraction for UI automation
- Zero external dependencies (Apple standard frameworks only)

## Architecture

### Hybrid Approach: XCUITest Runner + simctl

```
┌─────────────────────────────────────────────────────────┐
│               platform-ios (This Crate)                 │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────┐         ┌─────────────────┐      │
│  │ XCUITestClient  │         │  simctl Wrapper │      │
│  │  (HTTP/JSON)    │         │  (Command exec) │      │
│  └────────┬────────┘         └────────┬────────┘      │
│           │                           │                │
│           ▼                           ▼                │
│  ┌─────────────────┐         ┌─────────────────┐      │
│  │ localhost:8200   │         │ xcrun simctl    │      │
│  │ (XCUITest Runner│         │ boot/shutdown   │      │
│  └────────┬────────┘         └─────────────────┘      │
└───────────┼──────────────────────────────────────────┘
            │
            ▼
   ┌────────────────┐
   │ XCUITest Runner│ (Swift HTTP Server)
   └────────┬───────┘
            │
            ▼
   ┌────────────────┐
   │ iOS Simulator  │
   └────────────────┘
```

### Operation Routing

| Operation | Route | Reason |
|-----------|-------|--------|
| tap, swipe, long_press | XCUITest Runner | XCUITest API required |
| type, keypress, button | XCUITest Runner | XCUITest API required |
| accessibility info | XCUITest Runner | XCUITest API required |
| app launch/terminate | XCUITest Runner | XCUITest API required |
| screenshot | simctl | Direct simulator capture |
| boot/shutdown | simctl | Simulator lifecycle |
| install/uninstall | simctl | App management |
| clipboard | XCUITest Runner | UIPasteboard access |
| device list | simctl | Device discovery |

## Key Modules

### `xcuitest` - XCUITest Runner Communication

**Purpose**: HTTP/JSON communication with the XCUITest Runner Swift server.

**Main Types**:

```rust
pub struct XCUITestClient {
    base_url: String,
    http: reqwest::Client,
}

pub struct XCUITestRunner {
    project_path: PathBuf,
    destination: String,
    port: u16,
    process: Option<Child>,
}
```

**Sub-modules**:
- `xcuitest/client.rs` - HTTP client for XCUITest Runner
- `xcuitest/runner.rs` - xcodebuild process management
- `xcuitest/types.rs` - Request/Response serde types

**Example**:

```rust
use agent_mobile_platform_ios::xcuitest::XCUITestClient;

let client = XCUITestClient::default(); // localhost:8200

// Tap at coordinates
client.tap(100.0, 200.0).await?;

// Get accessibility tree
let tree = client.accessibility_info(true).await?;

// Launch an app
client.launch_app("com.example.app").await?;
```

### `simctl` - Simulator Lifecycle Management

**Purpose**: Direct wrapper around `xcrun simctl` for simulator operations.

**Main Functions**:

```rust
pub fn boot(udid: &str) -> Result<()>;
pub fn shutdown(udid: &str) -> Result<()>;
pub fn install(udid: &str, app_path: &str) -> Result<()>;
pub fn uninstall(udid: &str, bundle_id: &str) -> Result<()>;
pub fn list_simulators() -> Result<Vec<DeviceInfo>>;  // with TTL 5s cache
```

**Sub-modules**:
- `simctl/management.rs` - Simulator lifecycle operations
- `simctl/cache.rs` - DeviceCache (TTL 5s, invalidated on lifecycle operations)

### `snapshot` - Accessibility Tree Extraction

**Purpose**: Extract structured UI element data from iOS accessibility hierarchy.

**Process**:
1. Receive accessibility JSON from XCUITest Runner (HTTP GET)
2. Parse JSON accessibility elements
3. Extract frame, label, type, traits, value
4. Flatten hierarchy into list of interactive elements
5. Convert to `agent_mobile_core::RawElement` for snapshot command

## Usage Examples

### Controlling a Simulator

```rust
use agent_mobile_platform_ios::xcuitest::XCUITestClient;
use agent_mobile_platform_ios::simctl;

// Boot simulator
simctl::boot("UDID-123")?;

// Connect to XCUITest Runner
let client = XCUITestClient::default();

// Perform operations
client.tap(100.0, 200.0).await?;
client.type_text("Hello").await?;
client.launch_app("com.example.app").await?;

// Shutdown
simctl::shutdown("UDID-123")?;
```

### Managing Simulators

```rust
use agent_mobile_platform_ios::simctl;

// List available simulators
let devices = simctl::list_simulators()?;

// Boot a simulator
simctl::boot(&devices[0].udid)?;

// Install an app
simctl::install(&devices[0].udid, "/path/to/app.app")?;

// Shutdown
simctl::shutdown(&devices[0].udid)?;
```

## External Dependencies

### Xcode Command Line Tools

**Required for**:
- `xcrun simctl` commands (simulator management)
- Simulator runtime availability
- Building XCUITest Runner

**Installation**:
```bash
xcode-select --install
```

## Dependencies

**Core**:
- `agent-mobile-core` - Platform-agnostic types and traits
- `reqwest` - HTTP client for XCUITest Runner communication
- `tokio` 1.49 - Async runtime

**Utilities**:
- `serde` + `serde_json` - Serialization
- `thiserror` + `anyhow` - Error handling
- `image` - PNG/JPEG image conversion
- `tempfile` - Temporary file creation

## License

See the [main repository LICENSE](../../LICENSE) for details.

## Related Documentation

- [Main README](../../README.md) - CLI usage and installation
- [agent-mobile-core](../core/README.md) - Platform-agnostic types and traits
- [XCUITest Runner Architecture](../xcuitest-runner/ARCHITECTURE.md) - Swift server implementation
