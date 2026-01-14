# agent-ios

A Rust-based iOS device management tool that replaces Python components of idb with Rust, implementing gRPC client & CLI interface.

## Overview

This project is a Rust reimplementation of Facebook's [idb](https://github.com/facebook/idb) (iOS Development Bridge), replacing Python components with Rust for better performance and reliability.

## Features

- **Rust Implementation**: Core functionality rewritten in Rust
- **gRPC Client**: High-performance gRPC client for iOS device communication
- **CLI Interface**: Command-line interface built with Rust
- **Package Management**: Uses [mise](https://mise.jdx.dev/) for tool version management

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

## CI/CD

The project uses GitHub Actions for continuous integration:
- **Build**: Compiles the project and runs tests
- **Lint**: Checks code formatting and runs clippy

## License

See LICENSE file for details.