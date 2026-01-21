# agent-mobile CLI Architecture

## Overview

agent-mobile is a Rust-based iOS development bridge that provides both IDB-compatible commands and AI-optimized simplified commands for mobile automation.

### Tech Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Rust | 2021 edition | Main application |
| tokio | 1.49 | Async runtime |
| tonic + prost | 0.12 / 0.13 | gRPC client / Protocol Buffers |
| clap | 4.5 | CLI framework (derive) |
| idb_companion | - | Swift/ObjC daemon (external process) |

## Architecture Diagram

```text
┌─────────────────────────────────────────────────────────────┐
│  CLI Layer (src/cli/)                                        │
│  - Command parsing (clap derive)                             │
│  - IDB-compatible commands + AI-optimized commands          │
└─────────────────────────────────────────────────────────────┘
               ↓
┌─────────────────────────────────────────────────────────────┐
│  Companion Resolution (src/platform/ios/companion/)          │
│  - Read companion info from /tmp/idb/state                  │
│  - UDID → Address (DomainSocket | TCP)                      │
└─────────────────────────────────────────────────────────────┘
               ↓
┌─────────────────────────────────────────────────────────────┐
│  gRPC Client (src/platform/ios/grpc/)                        │
│  - IdbClient (Unix Domain Socket / TCP)                     │
│  - Streaming RPC support                                     │
└─────────────────────────────────────────────────────────────┘
               ↓
┌─────────────────────────────────────────────────────────────┐
│  idb_companion (Swift/ObjC external process)                 │
└─────────────────────────────────────────────────────────────┘
```

## CLI Structure

### Command Types

| Type | Description | Location |
|------|-------------|----------|
| **IDB-compatible** | `idb` subcommand (60+ commands) | `src/cli/idb/` |
| **Agent** | AI-optimized commands (7 commands) | `src/cli/agent/` |

### Agent Commands (Top-level, Simplified)

| Command | Description | Usage |
|---------|-------------|-------|
| `screenshot` | Take device screenshot | `agent-mobile screenshot <path>` |
| `tap` | Tap at screen coordinates | `agent-mobile tap <x> <y>` |
| `swipe` | Swipe gesture | `agent-mobile swipe <x1> <y1> <x2> <y2>` |
| `type` | Type text on device | `agent-mobile type <text>` |
| `launch` | Launch application | `agent-mobile launch <bundle_id>` |
| `install` | Install application | `agent-mobile install <bundle_path>` |
| `accessibility` | Get accessibility tree | `agent-mobile accessibility` |

### IDB Commands (`agent-mobile idb <command>`)

Full IDB CLI compatibility for advanced usage.

## Command Categories

| Category | Commands | Files |
|----------|----------|-------|
| **App Management** | launch, terminate, install, uninstall, list-apps | `cli/idb/launch.rs`, `cli/idb/terminate.rs`, etc. |
| **File Operations** | ls, mkdir, mv, rm, pull, push, read, write, tail | `cli/idb/file/` |
| **HID (Input)** | tap, swipe, text, button, key, key-sequence | `cli/idb/hid/` |
| **Target/Device** | list-targets, boot, shutdown, describe, connect | `cli/idb/target/`, `cli/idb/list_targets.rs` |
| **Media** | screenshot, video record/stream, media add | `cli/idb/screenshot.rs`, `cli/idb/video/` |
| **Accessibility** | accessibility-describe-all, accessibility-describe-point | `cli/idb/accessibility.rs` |
| **Testing** | xctest-install, xctest-list, xctest-run | `cli/idb/xctest_*.rs` |
| **Debug** | debugserver start/stop/status, dap | `cli/idb/debugserver/`, `cli/idb/dap.rs` |
| **Profiling** | instruments, xctrace record | `cli/idb/instruments.rs`, `cli/idb/xctrace.rs` |
| **Settings** | set, get, approve, revoke, location | `cli/idb/settings.rs`, `cli/idb/permissions.rs` |

## Data Flow

```text
1. CLI parsing (clap) ─────────────────────────────┐
   src/cli/mod.rs                                   │
                                                    ↓
2. Command handler ────────────────────────────────┤
   src/cli/idb/*.rs or src/cli/agent/*.rs          │
                                                    ↓
3. Companion resolution ───────────────────────────┤
   src/platform/ios/companion/resolver.rs          │
                                                    ↓
4. gRPC client connection ─────────────────────────┤
   src/platform/ios/grpc/client.rs                 │
                                                    ↓
5. Protocol Buffers serialization ─────────────────┤
   proto/idb.proto → tonic_build                   │
                                                    ↓
6. idb_companion (external) ───────────────────────┤
   Swift/ObjC daemon                               │
                                                    ↓
7. Response handling ──────────────────────────────┘
   JSON or human-readable output
```

## Key Components

| Component | File | Role |
|-----------|------|------|
| `IdbClient` | `grpc/client.rs` | gRPC connection management (UDS/TCP) |
| `CompanionResolver` | `companion/resolver.rs` | UDID → Address resolution |
| `CompanionState` | `companion/state.rs` | Read `/tmp/idb/state` |
| `CompanionSpawner` | `companion/spawner.rs` | Auto-start companion daemon |
| `with_client()` | `helpers/client.rs` | Command execution helper |

### with_client() Pattern

Most commands use the `with_client()` helper to reduce boilerplate:

```rust
pub async fn run(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.focus().await?;
        Ok(())
    }).await
}
```

## gRPC Streaming Types

| Type | Direction | Use Cases |
|------|-----------|-----------|
| **Unary** | Request → Response | screenshot, ls, describe |
| **Server-side** | Server → Client (stream) | file pull, log |
| **Client-side** | Client (stream) → Server | file push, HID events |
| **Bidirectional** | Both directions | launch (--wait-for) |

## Error Handling

### Connection Retry

- **Strategy**: Exponential backoff (1s, 2s, 4s...)
- **Max retries**: 3 attempts
- **Timeouts**: Connect 10s, Request 30s

```rust
// src/platform/ios/grpc/client.rs
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;
```

### Screenshot Retry

Framebuffer may not be ready immediately after simulator boot:
- **Max retries**: 5 attempts
- **Use case**: Wait for framebuffer initialization

## Directory Structure

```text
src/
├── cli/
│   ├── mod.rs              # CLI entry point, Commands enum
│   ├── agent/              # AI-optimized commands
│   │   ├── mod.rs
│   │   ├── screenshot.rs
│   │   ├── tap.rs
│   │   ├── swipe.rs
│   │   ├── type_text.rs
│   │   ├── launch.rs
│   │   ├── install.rs
│   │   └── accessibility.rs
│   ├── idb/                # IDB-compatible commands
│   │   ├── mod.rs          # IdbCommands enum
│   │   ├── file/           # File operations
│   │   ├── hid/            # Input operations
│   │   ├── target/         # Device management
│   │   ├── video/          # Video recording
│   │   ├── debugserver/    # Debug server
│   │   └── ...
│   └── helpers/            # Shared utilities
│       ├── client.rs       # with_client() helper
│       ├── output.rs       # Output formatting
│       └── ...
├── platform/
│   └── ios/
│       ├── companion/      # Companion management
│       │   ├── resolver.rs # UDID resolution
│       │   ├── state.rs    # State file parsing
│       │   ├── spawner.rs  # Auto-start daemon
│       │   └── lister.rs   # List companions
│       ├── grpc/           # gRPC client
│       │   ├── client.rs   # IdbClient
│       │   ├── app.rs      # App operations
│       │   ├── file.rs     # File operations
│       │   ├── hid.rs      # Input operations
│       │   ├── media.rs    # Screenshot/video
│       │   └── ...
│       ├── proto/          # Protocol Buffers
│       └── simctl/         # Simulator control
└── types.rs                # Shared types (Address, etc.)
```

## Address Types

```rust
pub enum Address {
    DomainSocket { path: String },
    Tcp { host: String, port: u16 },
}
```

- **DomainSocket**: Default for local simulators (faster)
- **TCP**: Used for remote devices or explicit configuration
