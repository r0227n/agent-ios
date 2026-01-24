# agent-mobile

> Mobile device automation CLI designed for AI agents

Natural language commands for iOS and Android automation. Minimize tokens, maximize expressiveness.

```bash
# 30 seconds to first automation
git clone https://github.com/your-org/agent-mobile.git
cd agent-mobile && cargo build --release
cargo install --path .

agent-mobile device boot "iPhone 15 Pro"
agent-mobile snapshot                    # See UI elements
agent-mobile tap @e1                     # Interact with elements
```

## Why agent-mobile?

- **Token-efficient**: Use `@e1` instead of complex selectors
- **AI-optimized**: Semantic locators (`find text "Login"`)
- **Cross-platform**: Unified API for iOS + Android
- **Session-aware**: Persistent state across commands

## Installation

### From Source (Recommended)

```bash
# 1. Clone repository
git clone https://github.com/your-org/agent-mobile.git
cd agent-mobile

# 2. Build release binary
cargo build --release

# 3. Install to PATH
cargo install --path .

# 4. Verify installation
agent-mobile --version
```

**Prerequisites**: Rust 1.92+ ([install via rustup](https://rustup.rs/))

### Platform-Specific Dependencies

**iOS Development**:
```bash
# Xcode Command Line Tools
xcode-select --install

# idb_companion (via Homebrew)
brew tap facebook/fb
brew install idb-companion
```

**Android Development**:
```bash
# Android Studio (recommended)
# Download from https://developer.android.com/studio

# Add to PATH
export ANDROID_HOME=$HOME/Library/Android/sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools

# Verify
adb version
```

See [CLAUDE.md](CLAUDE.md) for detailed development setup.

## Quick Start (5 Minutes)

### 1. Start a Device

```bash
# iOS: Boot a simulator
agent-mobile device boot "iPhone 15 Pro"

# Android: List connected devices/emulators
agent-mobile device list
```

### 2. Launch an App

```bash
# iOS example
agent-mobile app launch com.apple.mobilesafari

# Android example
agent-mobile app launch com.android.chrome
```

### 3. Capture UI State

```bash
agent-mobile snapshot
```

**Output example:**
```
@e1  Button       "Allow"          (200.0, 500.0)  [enabled]
@e2  TextField    "Search or URL"  (200.0, 100.0)  [enabled]
@e3  Button       "Cancel"         (300.0, 500.0)  [enabled]
```

### 4. Interact with Elements

```bash
# Method 1: Element references (token-efficient)
agent-mobile fill @e2 "https://example.com"
agent-mobile tap @e1

# Method 2: Semantic locators (descriptive)
agent-mobile find text "Allow" tap
agent-mobile find placeholder "Search or URL" fill "https://example.com"
```

### 5. Verify Results

```bash
# Wait for element
agent-mobile wait type StaticText text "Example Domain"

# Take screenshot
agent-mobile screenshot result.png
```

**Next**: See [Core Concepts](#core-concepts) to understand semantic locators and element references.

## Core Concepts

### Semantic Locators

Find elements by **meaning**, not implementation:

```bash
agent-mobile find text "Login"              # Visible text
agent-mobile find label "Email Address"     # Accessibility label
agent-mobile find placeholder "Enter email" # Placeholder
agent-mobile find type Button               # Element type

# Chain with actions
agent-mobile find text "Submit" tap
```

### Element References (`@e1`, `@e2`, ...)

Snapshot creates shorthand aliases for UI elements:

```bash
agent-mobile snapshot  # Generates @e1, @e2, @e3, ...
agent-mobile tap @e1   # Use in commands
agent-mobile get @e2 text  # Query properties
```

**Token savings:**
- Without refs: `find type TextField label "Email" --first fill "user@example.com"` (52 tokens)
- With refs: `fill @e2 "user@example.com"` (7 tokens)

**Lifecycle:** Valid until screen changes or new snapshot.

### Sessions

Manage multiple devices with isolated state:

```bash
agent-mobile session create ios-dev --udid ABC-123 -p ios
agent-mobile session create android-qa --udid emulator-5554 -p android

# Use via flag or environment variable
agent-mobile --session ios-dev snapshot
export AGENT_MOBILE_SESSION=ios-dev
```

### JSON Output

Every command supports `--json`:

```bash
agent-mobile snapshot --json | jq '.elements[] | select(.type == "Button")'
agent-mobile device list --json | jq '.[0].udid'
```

## Commands

### Core Automation

| Command | Description | Example |
|---------|-------------|---------|
| `tap` | Tap element/coordinates/key | `tap @e1` / `tap 100,200` / `tap home` |
| `fill` | Clear + type into field | `fill @e2 "text"` |
| `type` | Append to focused field | `type "more text"` |
| `check`/`uncheck` | Toggle checkboxes | `check @e1` |
| `select` | Choose picker value | `select @e1 "Option 2"` |
| `swipe` | Directional gesture | `swipe up` / `swipe left` |
| `scroll` | Scroll screen/element | `scroll down --element @e1` |

### Queries & State

| Command | Description | Example |
|---------|-------------|---------|
| `snapshot` | Capture UI + generate refs | `snapshot` |
| `screenshot` | Save PNG image | `screenshot output.png` |
| `get` | Get element property | `get @e1 text` |
| `is` | Check state (exit code) | `is @e1 enabled` |
| `wait` | Wait for condition | `wait @e1 --timeout 10` |
| `find` | Search + action | `find text "Login" tap` |

### Device & App

| Command | Description | Example |
|---------|-------------|---------|
| `device list` | List devices | `device list --json` |
| `device boot` | Start simulator (iOS) | `device boot "iPhone 15 Pro"` |
| `app launch` | Start application | `app launch com.example.app` |
| `app terminate` | Stop application | `app terminate com.example.app` |
| `app install` | Install .ipa/.apk | `app install app.ipa` |

### Session Management

| Command | Description | Example |
|---------|-------------|---------|
| `session create` | New session | `session create ios --udid ABC -p ios` |
| `session list` | Show sessions | `session list` |
| `session destroy` | Delete session | `session destroy ios` |

### Recording

| Command | Description | Example |
|---------|-------------|---------|
| `record` | Capture video (MP4) | `record output.mp4 --duration 30` |
| `console` | Stream device logs | `console --platform ios` |

### IDB Compatibility

80+ commands compatible with Facebook's IDB CLI. Examples:

```bash
agent-mobile idb list-targets
agent-mobile idb screenshot screenshot.png
agent-mobile idb install app.ipa
agent-mobile idb launch com.example.app
agent-mobile idb log
```

Full reference: [src/idb/README.md](src/idb/README.md)

**Tip**: Run `agent-mobile <command> --help` for detailed usage.

## Usage Examples

### Login Flow

```bash
# Launch app and capture state
agent-mobile app launch com.example.app
agent-mobile snapshot
# @e1  TextField       "Email"     ...
# @e2  SecureTextField "Password"  ...
# @e3  Button          "Login"     ...

# Fill credentials
agent-mobile fill @e1 "user@example.com"
agent-mobile fill @e2 "password123"
agent-mobile tap @e3

# Verify success
agent-mobile wait type Button text "Profile" --timeout 5
agent-mobile screenshot logged_in.png
```

### Form Submission with Validation

```bash
# Navigate and capture form
agent-mobile app launch com.example.app
agent-mobile find text "Sign Up" tap
agent-mobile snapshot

# Fill form
agent-mobile fill @e1 "John Doe"           # Name
agent-mobile fill @e2 "john@example.com"   # Email
agent-mobile select @e3 "United States"    # Country
agent-mobile check @e4                      # Terms checkbox
agent-mobile tap @e5                        # Submit

# Handle result
if agent-mobile is type StaticText text "Error" visible; then
    echo "Validation failed"
    agent-mobile screenshot error.png
    exit 1
fi
```

### Multi-Screen Navigation

```bash
# Navigate through app
agent-mobile app launch com.example.app

agent-mobile find text "Settings" tap
agent-mobile wait type NavigationBar text "Settings"

agent-mobile find text "Account" tap
agent-mobile wait type NavigationBar text "Account"

agent-mobile screenshot account_screen.png
```

### Cross-Platform Testing

```bash
# Setup sessions
agent-mobile session create ios --udid ABC-123 -p ios
agent-mobile session create android --udid emulator-5554 -p android

# Test function (works on both)
test_login() {
    local session=$1
    agent-mobile --session $session app launch com.example.app
    agent-mobile --session $session snapshot
    agent-mobile --session $session fill @e1 "user@example.com"
    agent-mobile --session $session fill @e2 "password"
    agent-mobile --session $session tap @e3
    agent-mobile --session $session screenshot ${session}_result.png
}

# Run tests
test_login ios
test_login android
```

### JSON Output for Automation

```bash
# Find enabled button
enabled_button=$(agent-mobile snapshot --json | \
    jq -r '.elements[] | select(.type == "Button" and .enabled == true) | .reference' | \
    head -n1)

agent-mobile tap "$enabled_button"

# Get device programmatically
udid=$(agent-mobile device list --json | jq -r '.[0].udid')
echo "Using device: $udid"
```

## Platform Details

### iOS

**Implementation**: idb_companion gRPC + xcrun simctl

**Supported**:
- Simulators (via simctl)
- Physical devices (via idb_companion)

**Device lifecycle**:
```bash
agent-mobile device boot "iPhone 15 Pro"   # Start simulator
agent-mobile device shutdown <UDID>        # Stop simulator
agent-mobile device create "Test iPhone" "iPhone 15 Pro" "iOS 17.0"
```

### Android

**Implementation**: ADB + UI Automator

**Supported**:
- Emulators (via ADB)
- Physical devices (via ADB)

**Requirements**:
- Android SDK Platform Tools
- USB debugging enabled (physical devices)

### Cross-Platform Compatibility

| Feature | iOS | Android | Notes |
|---------|-----|---------|-------|
| Semantic locators | ✓ | ✓ | Accessibility tree based |
| Element references | ✓ | ✓ | Snapshot required |
| Screenshots | ✓ | ✓ | PNG format |
| Video recording | ✓ | ✓ | MP4 format |
| Log streaming | ✓ | ✓ | Platform-specific formats |

## Documentation

- [IDB Commands Reference](src/idb/README.md) - 80+ IDB-compatible commands
- [Developer Guide](CLAUDE.md) - Development workflow (Japanese)
- [Crate Documentation](crates/) - Core, Platform, Gateway APIs

## Contributing

Contributions welcome! See [CLAUDE.md](CLAUDE.md) for development workflow.

**Before submitting a PR**:
- Test on actual devices (iOS/Android)
- Run `cargo test`
- Update relevant documentation

## License

MIT License. See [LICENSE](LICENSE) for details.

## Credits

Built with Rust, powered by [Facebook IDB](https://github.com/facebook/idb) and [Android ADB](https://developer.android.com/tools/adb).
