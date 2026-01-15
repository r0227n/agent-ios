# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

agent-mobile is a Rust reimplementation of Facebook's [idb](https://github.com/facebook/idb) (iOS Development Bridge) CLI. The project replaces Python components with Rust while maintaining compatibility with the existing `idb_companion` daemon (written in Swift/Objective-C).

**Architecture**: The original idb has two components:
- `idb_companion`: Swift/Objective-C daemon that communicates directly with iOS devices/simulators
- `idb` CLI: Python CLI that communicates with companion via gRPC

This project replaces only the Python CLI with Rust, maintaining full compatibility with the existing gRPC interface.

## When in Doubt

If you are uncertain or lack sufficient information, use AskUserQuestion to clarify with the user.

## Subagent Configuration

- When launching subagents via the Task tool, always set `run_in_background: true` to run them in the background by default
- Use TaskOutput to retrieve results when needed


## Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

## Architecture

### gRPC Communication Flow

```
agent-mobile idb CLI (Rust)
    ↓ gRPC (Unix socket or TCP)
idb_companion (Swift/ObjC)
    ↓
iOS Device/Simulator
```

The companion daemon maintains state in `/tmp/idb/state` (JSON format) with connection info for each target device.

### Module Structure

**Core modules:**
- `src/cli/` - Command-line interface using clap
  - `src/cli/idb/` - IDB command implementations (list-targets, launch, etc.)
- `src/grpc/` - gRPC client implementation using tonic
  - Generated code from `proto/idb.proto` via `build.rs`
  - `IdbClient` handles Unix socket and TCP connections to companion
- `src/companion/` - State file parsing and companion discovery
  - Reads `/tmp/idb/state` to find running companions
  - Maps UDIDs to connection addresses (socket path or TCP host/port)
- `src/types/` - Shared type definitions
  - `TargetDescription`: device/simulator information
  - `Address`: connection addressing (Unix socket or TCP)
- `src/simctl/` - Direct `xcrun simctl` integration
  - Fallback for listing simulators when companion isn't available
  - Parses simctl JSON output

### gRPC Connection Strategy

1. **With UDID specified**: Look up companion in `/tmp/idb/state` by UDID
2. **Without UDID**:
   - List all companions from state file
   - For simulators, also check `xcrun simctl` for additional targets
   - Connect to appropriate companion based on device type filters

### Protocol Buffers

`proto/idb.proto` defines the gRPC service interface. The proto file is compiled during build via `build.rs` using `tonic-build`, generating Rust code into the build output directory.

To add new RPC calls:
1. Update `proto/idb.proto` (or reference the complete proto in `idb/proto/idb.proto` submodule)
2. Add method to `src/grpc/client.rs`
3. Implement CLI command in `src/cli/idb/`

## Adding a New Command

Follow this pattern (reference existing commands like `list-targets` and `launch`):

1. **Add command variant** to `src/cli/idb/mod.rs`:
   ```rust
   pub enum IdbCommands {
       // ...
       YourCommand {
           #[arg(long)]
           your_arg: String,
       },
   }
   ```

2. **Create command implementation** in `src/cli/idb/your_command.rs`:
   ```rust
   pub async fn run(your_arg: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
       // Implementation
   }
   ```

3. **Add routing** in `src/main.rs`:
   ```rust
   match cli.command {
       Commands::Idb { command } => match command {
           // ...
           IdbCommands::YourCommand { your_arg } => {
               cli::idb::your_command::run(your_arg).await?;
           }
       }
   }
   ```

4. **If new gRPC calls needed**, add to `src/grpc/client.rs`:
   ```rust
   pub async fn your_method(&mut self) -> Result<Response, Error> {
       let request = tonic::Request::new(YourRequest { ... });
       let response = self.client.your_method(request).await?;
       Ok(response.into_inner())
   }
   ```

5. **Add integration test** in `tests/your_command_integration.rs`:
   ```rust
   #[test]
   fn test_your_command_vs_python_idb() {
       // Compare output with Python idb
   }
   ```

## Testing Strategy

**Unit tests**: Located alongside source code (standard Rust convention)
**Integration tests**: In `tests/` directory, compare behavior with Python idb CLI

Integration tests require:
- Python idb installed and in PATH
- At least one booted iOS simulator
- Binary built at `./target/debug/agent-mobile`

## Reference Implementation

The `idb/` directory is a git submodule containing the original Python implementation. Key reference files:

- `idb/idb/cli/commands/` - Python CLI command implementations
- `idb/idb/grpc/client.py` - Python gRPC client
- `idb/idb/common/format.py:215-253` - Output formatting logic
- `idb/idb/common/constants.py:27-30` - State file path constants
- `idb/idb/common/companion_set.py:52-94` - State file format parsing
- `idb/proto/idb.proto` - Complete gRPC service definition

## Build System

`build.rs` uses `tonic-build` to compile protocol buffers at build time. The generated code is included via `tonic::include_proto!("idb")` in `src/grpc/mod.rs`.
