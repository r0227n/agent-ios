# agent-mobile

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

AI-optimized mobile device automation CLI with semantic locators and token-efficient element references.

## Overview

**agent-mobile** is a Rust-based CLI tool for mobile device automation, specifically designed for AI agents like Claude Code. It provides a natural, intuitive interface for E2E testing and automation of iOS and Android apps, with features that minimize token usage while maximizing expressiveness.

**Key Design Goals**:
- **AI Agent Optimization**: Semantic locators and element references reduce token consumption
- **Cross-Platform**: Unified API for iOS (simulators + devices) and Android (emulators + devices)
- **Session Management**: Persistent state for multi-device workflows
- **JSON Output**: All commands support `--json` for programmatic parsing
- **IDB Compatibility**: 80+ commands compatible with Facebook's IDB CLI

## Key Features

### 🎯 Semantic Locators

Find elements by meaning, not technical selectors:

```bash
# Find by type
agent-mobile find type Button

# Find by text content
agent-mobile find text "Login"

# Find by accessibility label
agent-mobile find label "Email Address"

# Find by placeholder
agent-mobile find placeholder "Enter your email"

# Combine with actions
agent-mobile find text "Submit" tap
agent-mobile find label "Email" fill "test@example.com"
```

No need for XPath, CSS selectors, or complex accessibility IDs.

### 📍 Element References (`@e1`, `@e2`, ...)

Token-efficient shorthand for UI elements:

```bash
# Capture UI snapshot (generates references)
agent-mobile snapshot
# Output:
#   @e1  Button         "Login"           (100.0, 200.0)  [enabled]
#   @e2  TextField      "Email"           (100.0, 300.0)  [enabled]
#   @e3  SecureTextField "Password"       (100.0, 400.0)  [enabled]

# Use references (no need to repeat snapshot)
agent-mobile tap @e1
agent-mobile fill @e2 "user@example.com"
agent-mobile fill @e3 "password123"

# Get properties
agent-mobile get @e1 text    # → "Login"
agent-mobile is @e2 enabled  # → exit code 0 (true)
```

**Benefits**:
- **Token Efficiency**: `@e1` vs `find type Button text "Login" --first tap`
- **Stability**: References remain valid until screen changes
- **Readability**: Clear, sequential numbering

**Constraints**:
- References tied to last `snapshot` (invalidated on screen change)
- Off-screen elements have no reference
- Not persistent across sessions

### 🔄 Session Management

Manage multiple devices with persistent state:

```bash
# Create sessions for different devices
agent-mobile session create ios-dev --udid ABC-123 -p ios
agent-mobile session create android-test --udid emulator-5554 -p android

# Use sessions
agent-mobile --session ios-dev snapshot
agent-mobile --session android-test app launch com.example.app

# Or via environment variable
export AGENT_MOBILE_SESSION=ios-dev
agent-mobile snapshot  # Uses ios-dev session
```

**Session State Includes**:
- Platform (iOS/Android)
- Device UDID
- Last snapshot (element references)

### 🌐 Cross-Platform Support

| Platform | Simulators | Physical Devices | Implementation |
|----------|------------|------------------|----------------|
| **iOS** | ✓ | ✓ | idb_companion gRPC + simctl |
| **Android** | ✓ | ✓ | ADB + UI Automator |

**Auto-Detection**: `agent-mobile` automatically detects available devices when platform is not specified.

### 📊 JSON Output

Every command supports `--json` for programmatic parsing:

```bash
# Snapshot
agent-mobile snapshot --json | jq '.elements[] | select(.type == "Button")'

# Device list
agent-mobile device list --json | jq '.[] | .name'

# App list
agent-mobile app list --json | jq '.[] | .bundle_id'

# Get property
agent-mobile get @e1 text --json | jq '.value'
```

### 🔧 IDB CLI Compatibility

80+ commands compatible with Facebook's IDB for seamless migration:

```bash
agent-mobile idb list-targets
agent-mobile idb screenshot screenshot.png
agent-mobile idb install app.ipa
agent-mobile idb launch com.example.app
agent-mobile idb log
```

See [src/idb/README.md](src/idb/README.md) for full IDB command reference.

## Architecture

### High-Level Structure

```
┌─────────────────────────────────────────────────────────┐
│               agent-mobile CLI (Rust)                   │
│                (Command Dispatcher)                     │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│              agent-mobile-gateway                       │
│         (Platform-agnostic API layer)                   │
└─────────────┬────────────────────────┬──────────────────┘
              │                        │
      ┌───────▼───────┐        ┌──────▼───────┐
      │  platform-ios │        │platform-android│
      │ (gRPC+simctl) │        │  (ADB+XML)    │
      └───────┬───────┘        └──────┬───────┘
              │                       │
      ┌───────▼───────┐        ┌──────▼────────┐
      │ idb_companion │        │  adb server   │
      │ (Swift/ObjC)  │        │               │
      └───────┬───────┘        └──────┬────────┘
              │                       │
      ┌───────▼────────────────────────▼────────┐
      │    iOS Device / Simulator               │
      │    Android Device / Emulator            │
      └─────────────────────────────────────────┘
```

### Workspace Crates

```
agent-mobile/
├── crates/
│   ├── core/               # Platform-agnostic types and traits
│   ├── platform-ios/       # iOS implementation (gRPC + simctl)
│   ├── platform-android/   # Android implementation (ADB)
│   └── gateway/            # High-level unified API
├── src/                    # CLI implementation
│   ├── core/               # Core commands (tap, fill, snapshot)
│   ├── idb/                # IDB-compatible commands
│   ├── device/             # Device management
│   ├── app/                # App management
│   └── session/            # Session management
└── proto/                  # Protocol Buffer definitions
```

**Design Principles**:
- **Layered Architecture**: Clear separation between CLI, gateway, and platform layers
- **Platform Abstraction**: Core logic independent of iOS/Android specifics
- **Trait-Based**: Common operations defined as traits in `core`
- **Hybrid Approach**: Use best tool for each platform (gRPC for iOS, ADB for Android)

## Installation

### Prerequisites

**Required**:
- **Rust** 1.92+ (install via [rustup](https://rustup.rs/))

**Platform-Specific**:
- **iOS**: Xcode Command Line Tools + idb_companion
- **Android**: Android SDK Platform Tools (adb)

### Installing Xcode Tools (iOS)

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Verify simctl
xcrun simctl list devices
```

### Installing idb_companion (iOS)

```bash
# Via Homebrew (recommended)
brew tap facebook/fb
brew install idb-companion

# Or build from source
git clone https://github.com/facebook/idb.git
cd idb
./idb_build.sh idb_companion build
```

### Installing Android SDK (Android)

```bash
# Via Android Studio (recommended)
# Download from https://developer.android.com/studio

# Add to PATH (macOS/Linux)
export ANDROID_HOME=$HOME/Library/Android/sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools

# Verify adb
adb version
```

### Building agent-mobile

```bash
# Clone repository
git clone https://github.com/your-org/agent-mobile.git
cd agent-mobile

# Build release binary
cargo build --release

# Install to PATH
cargo install --path .

# Verify installation
agent-mobile --version
```

## Quick Start (5 Minutes)

### iOS Workflow

```bash
# 1. Boot a simulator
agent-mobile device boot "iPhone 15 Pro"

# 2. Launch an app
agent-mobile app launch com.example.app

# 3. Capture UI snapshot
agent-mobile snapshot
# Output:
#   @e1  Button       "Login"      (200.0, 500.0)  [enabled]
#   @e2  TextField    "Email"      (200.0, 300.0)  [enabled]
#   @e3  SecureTextField "Password" (200.0, 400.0)  [enabled]

# 4. Interact using element references
agent-mobile fill @e2 "test@example.com"
agent-mobile fill @e3 "password123"
agent-mobile tap @e1

# 5. Wait for navigation
agent-mobile wait type Button text "Dashboard"

# 6. Take screenshot for verification
agent-mobile screenshot result.png
```

### Android Workflow

```bash
# 1. List connected devices
agent-mobile device list

# 2. Launch an app
agent-mobile app launch com.example.app

# 3. Capture UI snapshot
agent-mobile snapshot
# Output:
#   @e1  Button       "LOGIN"         (540.0, 1200.0)  [clickable]
#   @e2  EditText     "Email address" (540.0, 800.0)   [clickable]
#   @e3  EditText     "Password"      (540.0, 1000.0)  [clickable]

# 4. Interact using semantic locators
agent-mobile find label "Email address" fill "test@example.com"
agent-mobile find label "Password" fill "password123"
agent-mobile find text "LOGIN" tap

# 5. Verify navigation
agent-mobile is type TextView text "Dashboard" visible  # exit code 0 = success

# 6. Take screenshot
agent-mobile screenshot result.png
```

## Command Categories

### Core Commands (AI Agent Focused)

These commands are optimized for AI agents with natural language patterns:

| Command | Purpose | Example |
|---------|---------|---------|
| `tap` | Tap element/coordinates/key | `tap @e1`, `tap 100,200`, `tap home` |
| `long-press` | Long press (default 1s) | `long-press @e1 --duration 2.0` |
| `fill` | Clear + type into field | `fill @e2 "test@example.com"` |
| `type` | Append to focused field | `type "additional text"` |
| `check` | Check checkbox/switch (idempotent) | `check @e1` |
| `uncheck` | Uncheck checkbox/switch (idempotent) | `uncheck @e1` |
| `select` | Select picker/spinner value | `select @e1 "Option 2"` |
| `swipe` | Swipe gesture | `swipe up`, `swipe left` |
| `scroll` | Scroll element/screen | `scroll down`, `scroll up --element @e1` |
| `get` | Get element property | `get @e1 text`, `get @e1 enabled` |
| `is` | Check element state (exit code) | `is @e1 enabled` (exit 0 = true) |
| `wait` | Wait for element | `wait @e1 --timeout 10` |
| `find` | Semantic locator search + action | `find text "Login" tap` |
| `snapshot` | Capture UI with references | `snapshot -i` (interactive only) |
| `screenshot` | Take screenshot | `screenshot output.png` |

### Device Management

```bash
# List devices
agent-mobile device list
agent-mobile device list --json

# Boot/shutdown simulator (iOS)
agent-mobile device boot "iPhone 15 Pro"
agent-mobile device shutdown <UDID>

# Create simulator (iOS)
agent-mobile device create "Test iPhone" "iPhone 15 Pro" "iOS 17.0"
```

### App Management

```bash
# Launch app
agent-mobile app launch com.example.app

# Terminate app
agent-mobile app terminate com.example.app

# Install app
agent-mobile app install app.ipa         # iOS
agent-mobile app install app.apk         # Android

# List installed apps
agent-mobile app list
agent-mobile app list --json | jq '.[] | .bundle_id'
```

### Session Management

```bash
# Create session
agent-mobile session create <name> --udid <UDID> --platform <ios|android>

# List sessions
agent-mobile session list

# Show session details
agent-mobile session show <name>

# Destroy session
agent-mobile session destroy <name>

# Use session
agent-mobile --session <name> snapshot
export AGENT_MOBILE_SESSION=<name>
```

### Recording

```bash
# Record video (MP4)
agent-mobile record output.mp4
agent-mobile record output.mp4 --duration 30

# Stream console logs
agent-mobile console
agent-mobile console --platform ios
agent-mobile console --platform android
```

### IDB Commands (80+ Available)

Full compatibility with Facebook's IDB CLI:

```bash
agent-mobile idb list-targets
agent-mobile idb screenshot screenshot.png
agent-mobile idb install app.ipa
agent-mobile idb launch com.example.app
agent-mobile idb terminate com.example.app
agent-mobile idb list-apps
agent-mobile idb log
agent-mobile idb accessibility
agent-mobile idb url "https://example.com"
agent-mobile idb notification "Test Title" "Test Body"
agent-mobile idb focus  # Bring simulator to front
# ... and 70+ more commands
```

See [src/idb/README.md](src/idb/README.md) for complete list.

## Usage Examples

### Login Flow Automation

```bash
# Start fresh
agent-mobile app launch com.example.app

# Capture initial screen
agent-mobile snapshot
# Output:
#   @e1  TextField       "Email"     (200.0, 300.0)  [enabled]
#   @e2  SecureTextField "Password"  (200.0, 400.0)  [enabled]
#   @e3  Button          "Login"     (200.0, 500.0)  [enabled]

# Fill credentials
agent-mobile fill @e1 "user@example.com"
agent-mobile fill @e2 "password123"

# Submit
agent-mobile tap @e3

# Wait for dashboard
agent-mobile wait type Button text "Profile"

# Verify success
agent-mobile screenshot logged_in.png
```

### Form Filling with Validation

```bash
# Navigate to form
agent-mobile app launch com.example.app
agent-mobile find text "Sign Up" tap

# Capture form elements
agent-mobile snapshot

# Fill form fields
agent-mobile fill @e1 "John Doe"
agent-mobile fill @e2 "john@example.com"
agent-mobile fill @e3 "555-1234"

# Select from dropdown
agent-mobile select @e4 "United States"

# Check terms checkbox
agent-mobile check @e5

# Submit
agent-mobile tap @e6

# Verify error or success
if agent-mobile is type StaticText text "Error" visible; then
    echo "Form validation failed"
    agent-mobile screenshot error.png
else
    echo "Form submitted successfully"
    agent-mobile screenshot success.png
fi
```

### Multi-Step Navigation

```bash
# Start app
agent-mobile app launch com.example.app

# Navigate through multiple screens
agent-mobile find text "Settings" tap
agent-mobile wait type NavigationBar text "Settings"
agent-mobile snapshot

agent-mobile find text "Account" tap
agent-mobile wait type NavigationBar text "Account"
agent-mobile snapshot

agent-mobile find text "Privacy" tap
agent-mobile wait type NavigationBar text "Privacy"
agent-mobile snapshot

# Take final screenshot
agent-mobile screenshot privacy_screen.png
```

### Screenshot-Based Verification

```bash
# Perform action
agent-mobile app launch com.example.app
agent-mobile find text "Login" tap

# Take before screenshot
agent-mobile screenshot before.png

# Interact
agent-mobile fill @e1 "test@example.com"
agent-mobile fill @e2 "password"
agent-mobile tap @e3

# Take after screenshot
agent-mobile screenshot after.png

# Compare externally (e.g., with ImageMagick)
compare before.png after.png diff.png
```

### Session-Based Multi-Device Testing

```bash
# Setup sessions
agent-mobile session create ios-15 --udid ABC-123 -p ios
agent-mobile session create android-12 --udid emulator-5554 -p android

# Test on iOS
agent-mobile --session ios-15 app launch com.example.app
agent-mobile --session ios-15 snapshot
agent-mobile --session ios-15 tap @e1
agent-mobile --session ios-15 screenshot ios_result.png

# Test on Android (parallel)
agent-mobile --session android-12 app launch com.example.app
agent-mobile --session android-12 snapshot
agent-mobile --session android-12 tap @e1
agent-mobile --session android-12 screenshot android_result.png
```

## Workspace Structure

The repository uses Cargo workspace with 4 crates:

```
agent-mobile/
├── crates/
│   ├── core/               # Platform-agnostic foundation
│   │   ├── src/
│   │   │   ├── error.rs          # Unified error types
│   │   │   ├── traits.rs         # Platform abstraction traits
│   │   │   ├── types/            # Common types (Platform, DeviceInfo)
│   │   │   ├── snapshot/         # UI element types
│   │   │   └── io/               # OutputWriter utilities
│   │   └── README.md
│   │
│   ├── platform-ios/       # iOS implementation
│   │   ├── src/
│   │   │   ├── grpc/             # IdbClient gRPC implementation
│   │   │   ├── companion/        # Daemon discovery & management
│   │   │   ├── simctl/           # xcrun simctl wrapper
│   │   │   ├── hid/              # HID event generation
│   │   │   ├── snapshot/         # Accessibility tree extraction
│   │   │   └── proto/            # Generated protobuf code
│   │   └── README.md
│   │
│   ├── platform-android/   # Android implementation
│   │   ├── src/
│   │   │   ├── adb/              # ADB command wrappers
│   │   │   │   ├── commands.rs   # Device listing, properties
│   │   │   │   ├── app.rs        # App management
│   │   │   │   ├── uiautomator.rs# UI hierarchy parsing
│   │   │   │   └── input.rs      # Input simulation
│   │   │   └── snapshot/         # Element extraction
│   │   └── README.md
│   │
│   └── gateway/            # High-level unified API
│       ├── src/
│       │   ├── platform.rs       # DeviceResolver (auto-detection)
│       │   ├── api/
│       │   │   ├── ios.rs        # IosDevice high-level API
│       │   │   └── android.rs    # AndroidDevice high-level API
│       │   └── console.rs        # Unified log streaming
│       └── README.md
│
├── src/                    # CLI implementation
│   ├── main.rs                   # Entry point, signal handling
│   ├── command.rs                # clap command definitions
│   ├── core/                     # Core commands (tap, fill, etc.)
│   ├── idb/                      # IDB-compatible commands
│   ├── device/                   # Device management
│   ├── app/                      # App management
│   └── session/                  # Session management
│
├── proto/                  # Protocol Buffer definitions
│   └── idb.proto                 # Facebook IDB gRPC service
│
└── tests/                  # Integration tests
    └── cli/                      # CLI integration tests
```

**Crate Documentation**:
- [agent-mobile-core](crates/core/README.md) - Platform-agnostic types and traits
- [agent-mobile-platform-ios](crates/platform-ios/README.md) - iOS implementation details
- [agent-mobile-platform-android](crates/platform-android/README.md) - Android implementation details
- [agent-mobile-gateway](crates/gateway/README.md) - High-level unified API

## Development Guide

### Building

```bash
# Debug build
cargo build --verbose

# Release build (optimized)
cargo build --release --verbose

# Build specific crate
cargo build -p agent-mobile-core
cargo build -p agent-mobile-platform-ios
```

### Testing

```bash
# Unit tests (all crates)
cargo test --verbose --bins

# Integration tests (requires running devices)
cargo test --test cli -- --test-threads=1

# Test specific crate
cargo test -p agent-mobile-core
```

### Development Workflow

When modifying features, follow this cycle:

1. **Implement**: Edit source files
2. **Build**: `cargo build --verbose`
3. **Unit Test**: `cargo test --verbose --bins`
4. **Device Test** (REQUIRED): Test on actual simulator/device
5. **Integration Test**: `cargo test --test cli -- --test-threads=1`
6. **Commit**: Only after successful device testing

**Important**: Never commit without device testing. Build success doesn't guarantee runtime correctness.

### iOS Device Testing

```bash
# Start test environment
/mobile-e2e ios

# Test your changes manually
agent-mobile device list
agent-mobile snapshot
# ... test the feature
```

### Android Device Testing

```bash
# Start test environment
/mobile-e2e android

# Test your changes manually
agent-mobile device list
agent-mobile snapshot
# ... test the feature
```

### Adding New Commands

See detailed guides:
- [CLAUDE.md](CLAUDE.md) - Development workflow and conventions
- [src/idb/README.md](src/idb/README.md) - Adding IDB-compatible commands
- [.claude/rules/cli-feature.md](.claude/rules/cli-feature.md) - Choosing gRPC vs simctl (iOS)

## Documentation

- **User Documentation**:
  - [README.md](README.md) - This file (overview, installation, usage)
  - [src/idb/README.md](src/idb/README.md) - IDB command architecture

- **Developer Documentation**:
  - [CLAUDE.md](CLAUDE.md) - Development guide (Japanese)
  - [crates/core/README.md](crates/core/README.md) - Core crate API
  - [crates/platform-ios/README.md](crates/platform-ios/README.md) - iOS implementation
  - [crates/platform-android/README.md](crates/platform-android/README.md) - Android implementation
  - [crates/gateway/README.md](crates/gateway/README.md) - Gateway API

- **Design Documentation**:
  - [.claude/rules/cli-feature.md](.claude/rules/cli-feature.md) - gRPC vs simctl decision guide

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Credits

**agent-mobile** is built on top of and inspired by:

- [Facebook IDB](https://github.com/facebook/idb) - iOS Development Bridge (Python implementation)
- [idb_companion](https://github.com/facebook/idb) - Swift/Objective-C daemon for iOS communication
- [Android Debug Bridge (ADB)](https://developer.android.com/tools/adb) - Android device communication
- [UI Automator](https://developer.android.com/training/testing/other-components/ui-automator) - Android UI testing framework

**Special Thanks**:
- Facebook for open-sourcing IDB and maintaining idb_companion
- The Rust community for excellent async and gRPC libraries
- Contributors to tonic, prost, clap, tokio, and other dependencies

## Contributing

Contributions are welcome! Please:

1. Read [CLAUDE.md](CLAUDE.md) for development workflow
2. Follow existing code style and conventions
3. Test on actual devices before submitting PR
4. Include integration tests for new features
5. Update documentation (READMEs, comments)

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/agent-mobile/issues)
- **Documentation**: See [docs/](docs/) directory
- **Examples**: See [examples/](examples/) directory (if available)

---

**Built with** ❤️ **using Rust, designed for AI agents**
