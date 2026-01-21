# agent-mobile

A Rust-based iOS device management tool that replaces Python components of idb with Rust, implementing gRPC client & CLI interface.

## Overview

This project is a Rust reimplementation of Facebook's [idb](https://github.com/facebook/idb) (iOS Development Bridge), replacing Python components with Rust for better performance and reliability.

## Features

- **Rust Implementation**: Core functionality rewritten in Rust
- **gRPC Client**: High-performance gRPC client for iOS device communication
- **CLI Interface**: Command-line interface built with Rust
- **Cross-Platform Support**: iOS and Android device management
- **Package Management**: Uses [mise](https://mise.jdx.dev/) for tool version management

## Quick Start

agent-mobile provides 5 main commands for mobile device interaction:

### HID Operations
Human Interface Device operations including touch gestures, keyboard input, and hardware buttons.

```bash
# Touch gestures
agent-mobile hid tap 100 200
agent-mobile hid tap center
agent-mobile hid swipe up
agent-mobile hid swipe 100,200,300,400
agent-mobile hid scroll down
agent-mobile hid long-press 150 250

# Keyboard input
agent-mobile hid text "Hello World"
agent-mobile hid key enter
agent-mobile hid key backspace
agent-mobile hid clear

# Hardware buttons
agent-mobile hid button home
agent-mobile hid button lock
agent-mobile hid button volume-up
```

### App Management
```bash
# Launch and terminate apps
agent-mobile app launch com.example.app
agent-mobile app terminate com.example.app

# Install and uninstall
agent-mobile app install app.ipa
agent-mobile app uninstall com.example.app
agent-mobile app list

# Permission management
agent-mobile app grant camera --bundle com.example.app
agent-mobile app revoke camera --bundle com.example.app
agent-mobile app reset camera --bundle com.example.app
```

### Device Management
```bash
# List devices
agent-mobile device list

# Boot and shutdown simulators
agent-mobile device boot "iPhone 15"
agent-mobile device shutdown <udid>

# Clipboard operations (iOS only)
agent-mobile device pbcopy "Hello"
agent-mobile device pbpaste
```

### iOS-Specific Features
```bash
# Full idb command compatibility
agent-mobile idb screenshot output.png
agent-mobile idb list-targets
agent-mobile idb accessibility describe-all

# See all 59 idb subcommands
agent-mobile idb --help
```

## Prerequisites

- Rust 1.75.0 or later
- [mise](https://mise.jdx.dev/) for managing dependencies

## Installation

Install mise (if not already installed):
```bash
curl https://mise.run | sh
```

Install project dependencies:
```bash
mise install
```

## Development

### Building

```bash
# Using mise
mise run build

# Or using cargo directly
cargo build
```

### Testing

```bash
# Using mise
mise run test

# Or using cargo directly
cargo test
```

### Linting

```bash
# Using mise
mise run lint

# Or using cargo directly
cargo clippy -- -D warnings
```

### Formatting

```bash
# Check formatting
mise run format-check

# Format code
mise run format
```

## Project Structure

- `src/` - Rust source code
- `idb/` - Git submodule of Facebook's idb project
- `.mise.toml` - Tool version and task configuration
- `.github/workflows/` - CI/CD configurations

## Known Issues

### Permission Approval for Photos, Camera, and Contacts

The `idb approve` command fails for `photos`, `camera`, and `contacts` permissions with a SQLite error from the `idb_companion` daemon:

```
table access has 17 columns but 13 values were supplied
```

**Root Cause**: SQLite schema mismatch in the upstream `idb_companion` daemon (Swift/Objective-C), not in the Rust implementation.

**Impact**:
- Both Python `idb` and Rust `agent-mobile` exhibit the same error
- `approve` operations fail for: `photos`, `camera`, `contacts`
- `revoke` operations work correctly for all permission types
- Other permissions work: `location`, `notification`, `url`, `microphone`

**Status**: Tests for affected permissions are marked with `#[ignore]` to prevent CI failures. See `docs/KNOWN_ISSUES.md` for detailed information.

### Screenshot "No Image available to encode"

The `screenshot` command may fail with `No Image available to encode` error when the simulator's framebuffer is not yet initialized.

**Root Cause**: Framebuffer initialization timing issue in the upstream `idb_companion` daemon (Swift/Objective-C), not in the Rust implementation.

**Solution**: agent-mobile includes automatic retry logic (3 attempts with exponential backoff). If retries fail:
1. Wait a moment after booting the simulator
2. Run `idb ui describe-all --udid <udid>` to trigger framebuffer initialization
3. Retry the screenshot command

**Status**: Mitigated with retry logic and test warmup helper. See `docs/KNOWN_ISSUES.md` for detailed information.

## Troubleshooting

For detailed information on known issues and their solutions, see:
- [docs/KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md) - Comprehensive list of known issues with technical details and workarounds

## CI/CD

The project uses GitHub Actions for continuous integration:
- **Build**: Compiles the project and runs tests
- **Lint**: Checks code formatting and runs clippy

## License

See LICENSE file for details.