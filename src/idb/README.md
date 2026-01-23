# IDB Command Implementation

This directory contains the Rust implementation of IDB (iOS Development Bridge) commands. It replaces the Python CLI from Facebook's [idb](https://github.com/facebook/idb) while maintaining full compatibility with the existing `idb_companion` daemon (Swift/Objective-C).

## Architecture

```
agent-mobile idb CLI (Rust)
    ↓ gRPC (Unix socket or TCP)
idb_companion (Swift/ObjC)
    ↓
iOS Device/Simulator
```

All commands in this directory communicate with `idb_companion` via gRPC to control iOS devices and simulators.

## Directory Structure

### Top-Level Commands (Single Files)

Each `.rs` file implements a standalone IDB command:

| File | Command | Description |
|------|---------|-------------|
| `accessibility.rs` | `agent-mobile idb accessibility` | Query accessibility information |
| `contacts.rs` | `agent-mobile idb contacts` | Manage simulator contacts |
| `crash.rs` | `idb crash` | List and manage crash logs |
| `dap.rs` | `agent-mobile idb dap` | Debug Adapter Protocol support |
| `dsym.rs` | `agent-mobile idb dsym` | Install debug symbols |
| `dylib.rs` | `agent-mobile idb dylib` | Install dynamic libraries |
| `focus.rs` | `agent-mobile idb focus` | Bring simulator window to front |
| `framework.rs` | `agent-mobile idb framework` | Install frameworks |
| `install.rs` | `agent-mobile idb install` | Install applications |
| `instruments.rs` | `agent-mobile idb instruments` | Run Instruments profiling |
| `keychain.rs` | `agent-mobile idb keychain` | Manage keychain data |
| `kill.rs` | `agent-mobile idb kill` | Kill the idb daemon |
| `launch.rs` | `agent-mobile idb launch` | Launch applications |
| `list_apps.rs` | `agent-mobile idb list-apps` | List installed applications |
| `list_targets.rs` | `agent-mobile idb list-targets` | List connected devices/simulators |
| `location.rs` | `agent-mobile idb location` | Set device location |
| `log.rs` | `agent-mobile idb log` | Stream device logs |
| `memory.rs` | `agent-mobile idb memory-warning` | Simulate memory warnings |
| `notification.rs` | `agent-mobile idb notification` | Send push notifications |
| `permissions.rs` | `agent-mobile idb permissions` | Manage app permissions |
| `photos.rs` | `agent-mobile idb photos` | Manage photo library |
| `screenshot.rs` | `agent-mobile idb screenshot` | Take device screenshots |
| `settings.rs` | `agent-mobile idb settings` | Manage device settings |
| `shell.rs` | `agent-mobile idb shell` | Execute shell commands |
| `terminate.rs` | `agent-mobile idb terminate` | Terminate running apps |
| `uninstall.rs` | `agent-mobile idb uninstall` | Uninstall applications |
| `url.rs` | `agent-mobile idb url` | Open URLs on device |
| `xctest_install.rs` | `agent-mobile idb xctest-install` | Install XCTest bundles |
| `xctest_list.rs` | `agent-mobile idb xctest-list` | List available tests |
| `xctest_list_bundle.rs` | `agent-mobile idb xctest-list-bundle` | List tests in a bundle |
| `xctest_run.rs` | `agent-mobile idb xctest-run` | Run XCTest suites |
| `xctrace.rs` | `agent-mobile idb xctrace` | Record performance traces |

## Adding a New Command

To add a new IDB command:

1. **Create implementation file** in this directory:
   ```rust
   // src/idb/your_command.rs
   pub async fn run(/* args */) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
       // Implementation
   }
   ```

2. **Add module declaration** to `mod.rs`:
   ```rust
   pub mod your_command;
   ```

3. **Add command variant** to `IdbCommands` enum in `mod.rs`:
   ```rust
   pub enum IdbCommands {
       // ...
       YourCommand {
           #[arg(long)]
           your_arg: String,
       },
   }
   ```

4. **Add routing** in `src/main.rs`:
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

5. **If new gRPC calls needed**, add to `src/grpc/client.rs` (see `proto/idb.proto` for available methods)

6. **Add integration test** in `tests/idb/your_command_integration.rs` comparing output with Python `idb`

## Testing

All IDB commands have integration tests located in `tests/idb/`:
- Compare behavior with the original Python `idb` CLI
- Tests require a booted iOS simulator and Python `idb` installed
- Run with: `cargo test --test idb`

See the [project README](../../../README.md) for complete testing instructions.

## Reference Implementation

The original Python implementation is available in the `idb/` submodule for reference:
- Command implementations: `idb/idb/cli/commands/`
- gRPC client: `idb/idb/grpc/client.py`
- Output formatting: `idb/idb/common/format.py`

## Key Dependencies

- **gRPC communication**: `src/grpc/` - Client implementation using tonic
- **iOS platform integration**: `src/platform/ios/` - Companion, simctl, device management
- **Shared types**: `src/types/` - Common data structures
- **Protocol definitions**: `proto/idb.proto` - gRPC service definitions (compiled at build time)
