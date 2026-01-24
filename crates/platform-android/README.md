# agent-mobile-platform-android

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

Android platform implementation for agent-mobile, providing device and emulator automation via ADB and UI Automator.

## Overview

`agent-mobile-platform-android` implements Android-specific functionality for the agent-mobile workspace. It uses a **command-based architecture** that wraps the Android Debug Bridge (ADB) for device communication and UI Automator for UI hierarchy parsing.

**Key Features**:
- Direct ADB command integration for device operations
- UI Automator XML parsing for element discovery
- Device and emulator detection
- App lifecycle management (launch, terminate, install)
- Platform property queries (API level, Android version)
- Element finding by text, ID, and type

**Architecture Differences from iOS**:
- No gRPC daemon (unlike iOS's idb_companion)
- Synchronous command execution via `adb` CLI
- XML-based UI hierarchy (unlike iOS's native accessibility API)
- Limited streaming capabilities (no real-time log streaming like iOS)

## Architecture

### ADB-Based Approach

```
┌─────────────────────────────────────────────────────────┐
│          platform-android (This Crate)                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────────────────────────────────┐      │
│  │        ADB Command Wrappers                 │      │
│  │  (tokio::process::Command)                  │      │
│  └──────────────────┬──────────────────────────┘      │
│                     │                                  │
│                     ▼                                  │
│  ┌─────────────────────────────────────────────┐      │
│  │   adb shell / adb devices / adb install     │      │
│  │   adb pull / adb push / uiautomator dump    │      │
│  └──────────────────┬──────────────────────────┘      │
└────────────────────┼──────────────────────────────────┘
                     │
                     ▼
         ┌───────────────────────┐
         │   ADB Server (adbd)   │
         └───────────┬───────────┘
                     │
                     ▼
         ┌───────────────────────┐
         │  Android Device or    │
         │      Emulator         │
         └───────────────────────┘
```

### UI Automator XML Parsing

UI hierarchy is obtained through a two-step process:

1. **Dump**: `adb shell uiautomator dump /sdcard/window_dump.xml`
2. **Retrieve**: `adb shell cat /sdcard/window_dump.xml`
3. **Parse**: XML → `AccessibilityElement` tree

This approach is simpler than iOS's native accessibility API but provides similar element information.

## Key Modules

### `adb` - Device Management and Command Execution

**Purpose**: Core ADB command wrappers for device communication.

**Main Types and Functions**:

```rust
pub enum AdbError {
    CommandFailed(String),
    ExecutionError(std::io::Error),
    AdbNotFound,
    InvalidOutput(String),
}

pub type Result<T> = std::result::Result<T, AdbError>;

// Device listing
pub fn list_devices() -> Result<Vec<(String, String)>>;
pub fn list_avds() -> Result<Vec<String>>;

// Platform detection
pub fn is_adb_available() -> bool;
pub fn is_emulator(serial: &str) -> Result<bool>;
pub fn get_avd_name(serial: &str) -> Result<Option<String>>;

// Device properties
pub fn get_api_level(serial: &str) -> Result<Option<u32>>;
pub fn get_android_version(serial: &str) -> Result<Option<String>>;
pub fn get_device_model(serial: &str) -> Result<Option<String>>;
```

**Features**:
- Automatic ADB availability detection
- Device serial number management
- Emulator vs physical device detection
- Property queries via `adb shell getprop`

**Example**:

```rust
use agent_mobile_platform_android::{
    list_devices, is_emulator, get_api_level, get_android_version
};

// List all connected devices
let devices = list_devices()?;
for (serial, state) in devices {
    println!("Device: {} ({})", serial, state);

    // Check if emulator
    if is_emulator(&serial)? {
        println!("  Type: Emulator");
    } else {
        println!("  Type: Physical device");
    }

    // Get platform info
    if let Some(api) = get_api_level(&serial)? {
        println!("  API Level: {}", api);
    }

    if let Some(version) = get_android_version(&serial)? {
        println!("  Android Version: {}", version);
    }
}
```

### `adb::app` - Application Management

**Purpose**: App lifecycle operations (launch, terminate, install, list).

**Main Types**:

```rust
pub struct AppInfo {
    pub package_name: String,
    pub version_name: Option<String>,
    pub version_code: Option<u32>,
    pub path: Option<String>,
}

pub async fn launch(serial: Option<&str>, package_name: &str) -> Result<()>;
pub async fn launch_activity(serial: Option<&str>, package: &str, activity: &str) -> Result<()>;
pub async fn terminate(serial: Option<&str>, package_name: &str) -> Result<()>;
pub async fn install(serial: Option<&str>, apk_path: &str) -> Result<()>;
pub async fn uninstall(serial: Option<&str>, package_name: &str) -> Result<()>;
pub async fn list_packages(serial: Option<&str>) -> Result<Vec<String>>;
```

**Launch Strategy**:
- Uses `adb shell monkey` for simple app launches (finds launcher activity automatically)
- Uses `adb shell am start` for specific activity launches

**Example**:

```rust
use agent_mobile_platform_android::adb::app::{launch, terminate, install, list_packages};

// Launch an app
launch(Some("emulator-5554"), "com.example.app").await?;

// Launch specific activity
launch_activity(Some("emulator-5554"), "com.example.app", ".MainActivity").await?;

// Terminate
terminate(Some("emulator-5554"), "com.example.app").await?;

// Install APK
install(Some("emulator-5554"), "/path/to/app.apk").await?;

// List installed packages
let packages = list_packages(Some("emulator-5554")).await?;
for package in packages {
    println!("Package: {}", package);
}
```

### `adb::uiautomator` - UI Hierarchy Parsing

**Purpose**: Extract and parse Android UI hierarchy using UIAutomator.

**Main Types**:

```rust
pub struct AccessibilityElement {
    pub text: Option<String>,
    pub content_desc: Option<String>,
    pub resource_id: Option<String>,
    pub class: Option<String>,
    pub package: Option<String>,
    pub bounds: Option<[i32; 4]>,  // [left, top, right, bottom]
    pub clickable: bool,
    pub scrollable: bool,
    pub enabled: bool,
    pub focused: bool,
    pub selected: bool,
    pub children: Vec<AccessibilityElement>,
}

pub async fn dump_ui(serial: Option<&str>) -> Result<String>;
pub fn parse_ui_hierarchy(xml: &str) -> Result<Vec<AccessibilityElement>>;
pub fn find_by_text(elements: &[AccessibilityElement], text: &str) -> Vec<&AccessibilityElement>;
pub fn find_by_id(elements: &[AccessibilityElement], id: &str) -> Vec<&AccessibilityElement>;
pub fn find_by_type(elements: &[AccessibilityElement], class: &str) -> Vec<&AccessibilityElement>;
```

**Element Properties**:
- `text` - Visible text content
- `content_desc` - Accessibility label (like iOS VoiceOver)
- `resource_id` - Android resource ID (e.g., "com.example:id/login_button")
- `class` - Widget class (e.g., "android.widget.Button")
- `bounds` - Screen coordinates for interaction

**Helper Methods**:

```rust
impl AccessibilityElement {
    pub fn center(&self) -> Option<(f64, f64)>;
    pub fn label(&self) -> Option<&str>;
    pub fn element_type(&self) -> Option<&str>;
}
```

**Example**:

```rust
use agent_mobile_platform_android::{dump_ui, parse_ui_hierarchy, find_by_text, find_by_id};

// Dump UI hierarchy
let xml = dump_ui(Some("emulator-5554")).await?;

// Parse to elements
let elements = parse_ui_hierarchy(&xml)?;

// Find element by text
let login_buttons = find_by_text(&elements, "Login");
if let Some(button) = login_buttons.first() {
    if let Some((x, y)) = button.center() {
        println!("Login button at ({}, {})", x, y);
    }
}

// Find element by resource ID
let email_fields = find_by_id(&elements, "com.example:id/email_input");
if let Some(field) = email_fields.first() {
    println!("Email field: {:?}", field);
}

// Find all buttons
let buttons = find_by_type(&elements, "Button");
println!("Found {} buttons", buttons.len());
```

### `snapshot` - Element Extraction

**Purpose**: Convert Android UI hierarchy to agent-mobile core types.

**Main Function**:

```rust
pub fn extract_android_elements(
    elements: &[AccessibilityElement]
) -> Vec<agent_mobile_core::RawElement>;
```

**Conversion Process**:
1. Traverse `AccessibilityElement` tree
2. Convert bounds to `Frame` (x, y, width, height)
3. Map Android class names to element types (Button, EditText, etc.)
4. Extract text, content_desc as label
5. Determine interactivity (clickable, enabled)
6. Return flattened `RawElement` list for snapshot command

**Example**:

```rust
use agent_mobile_platform_android::{dump_ui, parse_ui_hierarchy, extract_android_elements};

// Capture UI hierarchy
let xml = dump_ui(Some("emulator-5554")).await?;
let elements = parse_ui_hierarchy(&xml)?;

// Extract for snapshot
let raw_elements = extract_android_elements(&elements);

// Filter interactive elements
let interactive: Vec<_> = raw_elements.iter()
    .filter(|e| e.is_interactive())
    .collect();

println!("Found {} interactive elements", interactive.len());
for (i, elem) in interactive.iter().enumerate() {
    println!("@e{}: {} at ({}, {})",
        i,
        elem.element_type,
        elem.frame.x,
        elem.frame.y
    );
}
```

## Usage Examples

### Listing Devices

```rust
use agent_mobile_platform_android::{list_devices, list_avds, is_emulator, get_device_model};

// List connected devices
let devices = list_devices()?;
println!("Connected devices:");
for (serial, state) in &devices {
    println!("  {} ({})", serial, state);

    if let Ok(true) = is_emulator(serial) {
        println!("    Type: Emulator");
    }

    if let Ok(Some(model)) = get_device_model(serial) {
        println!("    Model: {}", model);
    }
}

// List available AVDs
let avds = list_avds()?;
println!("\nAvailable AVDs:");
for avd in avds {
    println!("  {}", avd);
}
```

### Dumping UI Hierarchy

```rust
use agent_mobile_platform_android::{dump_ui, parse_ui_hierarchy};

// Dump and parse UI
let xml = dump_ui(Some("emulator-5554")).await?;
let elements = parse_ui_hierarchy(&xml)?;

println!("UI Elements:");
for elem in &elements {
    if let Some(label) = elem.label() {
        println!("  {}: {}", elem.element_type().unwrap_or("Unknown"), label);
    }
}
```

### Finding Elements

```rust
use agent_mobile_platform_android::{
    dump_ui, parse_ui_hierarchy, find_by_text, find_by_id, find_by_type
};

let xml = dump_ui(Some("emulator-5554")).await?;
let elements = parse_ui_hierarchy(&xml)?;

// By text
let submit_buttons = find_by_text(&elements, "Submit");

// By resource ID
let email_fields = find_by_id(&elements, "com.example:id/email");

// By type (all buttons)
let all_buttons = find_by_type(&elements, "Button");

println!("Found {} submit buttons", submit_buttons.len());
println!("Found {} email fields", email_fields.len());
println!("Found {} buttons total", all_buttons.len());
```

### Platform Detection

```rust
use agent_mobile_platform_android::{is_emulator, get_avd_name, get_api_level};

let serial = "emulator-5554";

if is_emulator(serial)? {
    if let Some(avd_name) = get_avd_name(serial)? {
        println!("Running on emulator: {}", avd_name);
    }

    if let Some(api_level) = get_api_level(serial)? {
        println!("API Level: {}", api_level);
    }
} else {
    println!("Physical device");
}
```

## External Dependencies

### Android SDK

**Required for**: `adb` command-line tool, emulator management

**Installation**:
```bash
# Via Android Studio (recommended)
# Download from https://developer.android.com/studio

# Or via command-line tools
# Download from https://developer.android.com/studio#command-tools
```

**Environment Setup**:
```bash
# Add to PATH (macOS/Linux)
export ANDROID_HOME=$HOME/Library/Android/sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools
export PATH=$PATH:$ANDROID_HOME/emulator

# Verify installation
adb version
```

### `adb` in PATH

**Verification**:
```bash
$ adb version
Android Debug Bridge version 1.0.41
```

**Common Issues**:
- **"adb not found"**: Install Android SDK platform-tools
- **"no devices"**: Enable USB debugging on device, or start emulator
- **"unauthorized"**: Accept "Allow USB debugging" prompt on device

## Limitations

Unlike the iOS implementation, this Android crate has some architectural limitations:

| Feature | iOS | Android | Reason |
|---------|-----|---------|--------|
| Real-time Log Streaming | ✓ | ✗ | No gRPC streaming; use `adb logcat` separately |
| Install Progress | ✓ | ✗ | `adb install` doesn't provide progress callbacks |
| Type Safety | ✓ | △ | ADB output is text-based, requires parsing |
| Connection Pooling | ✓ | ✗ | ADB server handles connections, not controllable |
| Native API Access | ✓ | ✗ | XML-based UI hierarchy vs native accessibility API |

**Workarounds**:
- **Log streaming**: Run `adb logcat` in separate process, parse output
- **Install progress**: Use `adb install` with timeout, no granular progress
- **Type safety**: Careful output parsing with error handling

## Dependencies

**Core**:
- `agent-mobile-core` - Platform-agnostic types and traits
- `tokio` 1.49 - Async runtime with process support

**Utilities**:
- `serde` + `serde_json` - Serialization
- `thiserror` + `anyhow` - Error handling
- `regex` - Pattern matching for ADB output parsing

## License

See the [main repository LICENSE](../../LICENSE) for details.

## Related Documentation

- [Main README](../../README.md) - CLI usage and installation
- [agent-mobile-core](../core/README.md) - Platform-agnostic types and traits
- [agent-mobile-platform-ios](../platform-ios/README.md) - iOS implementation
- [Android ADB Documentation](https://developer.android.com/tools/adb) - Official ADB reference
- [UI Automator](https://developer.android.com/training/testing/other-components/ui-automator) - UI testing framework
