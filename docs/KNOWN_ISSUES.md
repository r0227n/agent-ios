# Known Issues

This document describes known issues in agent-mobile and their root causes.

## Permission Approval Failures for Photos, Camera, and Contacts

### Summary

The `idb approve` command fails for specific permission types (`photos`, `camera`, `contacts`) with a SQLite schema mismatch error from the `idb_companion` daemon.

### Error Message

```
table access has 17 columns but 13 values were supplied
```

### Affected Commands

- `idb approve <bundle_id> photos`
- `idb approve <bundle_id> camera`
- `idb approve <bundle_id> contacts`

### Root Cause

This is **not a bug in agent-mobile** (Rust implementation). The error originates from the upstream `idb_companion` daemon (written in Swift/Objective-C), which is responsible for:
- Managing iOS simulator permissions via TCC (Transparency, Consent, and Control) database
- Executing SQLite INSERT statements to grant permissions

The companion daemon attempts to insert 13 values into a SQLite table that expects 17 columns, indicating a schema version mismatch or missing default values.

### Evidence that this is not a Rust Bug

1. **Python idb exhibits the same error**: When running the original Python implementation of `idb approve <bundle_id> photos`, the exact same error occurs.
2. **gRPC layer is working correctly**: The Rust gRPC client successfully communicates with the companion daemon and receives error responses.
3. **Revoke operations work**: All `idb revoke` commands work correctly for all permission types, demonstrating that the gRPC communication and Rust implementation are functioning properly.
4. **Other permissions work**: Permissions like `location`, `notification`, `url`, and `microphone` can be approved successfully.

### Impact

| Permission Type | `approve` Status | `revoke` Status |
|-----------------|------------------|-----------------|
| `photos` | ❌ Fails | ✅ Works |
| `camera` | ❌ Fails | ✅ Works |
| `contacts` | ❌ Fails | ✅ Works |
| `location` | ✅ Works | ✅ Works |
| `notification` | ✅ Works | ✅ Works |
| `url` | ✅ Works | ✅ Works |
| `microphone` | ✅ Works | ✅ Works |

### Working Alternatives

#### Manual Permission Setting via simctl

For simulator testing, you can use `xcrun simctl` directly to grant permissions:

```bash
# Grant photos permission
xcrun simctl privacy <udid> grant photos <bundle_id>

# Grant camera permission
xcrun simctl privacy <udid> grant camera <bundle_id>

# Grant contacts permission
xcrun simctl privacy <udid> grant contacts <bundle_id>
```

#### Using Working Permission Types

For testing permission workflows, consider using permission types that work correctly:

```bash
# These work with both approve and revoke
idb approve <bundle_id> location --udid <udid>
idb approve <bundle_id> notification --udid <udid>
idb approve <bundle_id> url --scheme https --udid <udid>
```

### Technical Details

#### SQLite Schema Investigation

The TCC database schema varies between iOS/macOS versions. The `access` table in recent versions includes columns such as:
- `service` (TEXT)
- `client` (TEXT)
- `client_type` (INTEGER)
- `auth_value` (INTEGER)
- `auth_reason` (INTEGER)
- `auth_version` (INTEGER)
- `csreq` (BLOB)
- And potentially 10 more columns added in recent versions

The companion daemon's INSERT statement appears to use an older schema format with 13 columns, while the actual database expects 17 columns.

#### gRPC Request/Response Flow

1. **Rust client** → sends `ApproveRequest` via gRPC
2. **Companion daemon** → receives request, attempts SQLite INSERT
3. **SQLite** → rejects INSERT due to column count mismatch
4. **Companion daemon** → returns error in gRPC response
5. **Rust client** → receives error, propagates to user

### Future Resolution

This issue requires a fix in the upstream `idb_companion` daemon:
- Update SQLite INSERT statements to match current TCC schema
- Add proper default values for new columns
- Add version detection for different iOS/macOS versions

**Repository**: https://github.com/facebook/idb
**Component**: `idb_companion` (Swift/Objective-C code)

### Test Status

Integration tests for affected permissions are marked with `#[ignore]` attribute to prevent CI failures:

```rust
#[test]
#[ignore = "idb_companion SQLite schema mismatch: table 'access' expects 17 columns but receives 13"]
fn test_approve_photos() { ... }
```

To run these tests explicitly (they will fail as expected):

```bash
# Run ignored tests
cargo test --test permissions_integration -- --ignored

# Run all tests including ignored
cargo test --test permissions_integration -- --include-ignored
```

### Related Files

- `src/cli/idb/permissions.rs` - Permission command implementation
- `src/grpc/client.rs` - gRPC client that communicates with companion
- `tests/permissions_integration.rs` - Integration tests (6 tests ignored)
- `proto/idb.proto` - gRPC protocol definition
- `README.md` - User-facing known issues section
- `todo.md` - Project checklist with issue markers

### References

- [idb GitHub Repository](https://github.com/facebook/idb)
- [Apple TCC Documentation](https://developer.apple.com/documentation/bundleresources/privacy_manifest_files)
- [SQLite Column Count Error](https://www.sqlite.org/lang_insert.html)

---

## Screenshot: "No Image available to encode"

### Summary

The `screenshot` command may fail with `No Image available to encode` error when the simulator's framebuffer is not yet initialized.

### Error Message

```
No Image available to encode
```

### Affected Commands

- `idb screenshot <dest_path> --udid <udid>`
- `idb screenshot -` (stdout output)

### Root Cause

This is **not a bug in agent-mobile** (Rust implementation). The error originates from the upstream `idb_companion` daemon when it cannot acquire a valid CGImage from the simulator's framebuffer/IOSurface.

The framebuffer/IOSurface initialization occurs when:
- The simulator is fully booted (state: "Booted")
- The display/framebuffer service has initialized
- At least one frame has been rendered to the surface
- A consumer has attached to the framebuffer

**Key Finding**: Accessing the simulator's UI (e.g., via `idb ui describe-all`) triggers framebuffer initialization, after which screenshots work reliably.

### Evidence that this is not a Rust Bug

1. **Python idb exhibits the same error**: When running the original Python implementation of `idb screenshot`, the exact same error occurs under the same conditions.
2. **gRPC layer is working correctly**: The Rust gRPC client successfully communicates with the companion daemon and receives error responses.
3. **Error originates in companion**: The error occurs in `/idb/FBSimulatorControl/Framebuffer/FBSimulatorImage.m:104-110` when `CGImageRef` is `nil`.

### Solution

The CLI includes **automatic retry logic** (3 attempts with exponential backoff: 500ms, 1000ms, 2000ms). If the automatic retries fail:

1. **Wait a moment**: Give the simulator a few seconds after boot
2. **Interact with UI**: Run `idb ui describe-all --udid <udid>` to trigger framebuffer initialization
3. **Open Simulator.app**: Ensure the simulator window is visible and active
4. **Retry the command**: The framebuffer should be ready after initialization

### Example

```bash
# Trigger framebuffer initialization
idb ui describe-all --udid <udid>

# Take screenshot (should succeed now)
idb screenshot /tmp/screenshot.png --udid <udid>
```

### Technical Details

#### Error Chain

```
agent-mobile (Rust) screenshot command
    ↓
src/grpc/client.rs::screenshot() [unary RPC]
    ↓
idb_companion gRPC service
    ↓
FBSimulatorScreenshotCommands.takeScreenshot()
    ↓
FBSimulatorImage.image property getter
    ↓
FBSurfaceImageGenerator.image (CGImageRef or nil)
    ↓
FBSimulatorImage.pngImageDataWithError()
    ↓
FBSimulatorImage.imageDataFromImage() - ERROR if image is nil
    ↓
"No Image available to encode" error back to agent-mobile
```

#### Retry Logic Implementation

The Rust implementation includes retry logic that Python idb lacks:

- **Max retries**: 3 attempts
- **Backoff**: Exponential (500ms, 1000ms, 2000ms)
- **Retry condition**: Only retries on framebuffer errors
- **User feedback**: Progress messages during retries
- **Enhanced error message**: Provides actionable suggestions after exhausting retries

### Test Strategy

Integration tests use a **test warmup helper** (`ensure_framebuffer_ready()` in `tests/common/mod.rs`) to proactively initialize the framebuffer before screenshot tests:

```rust
#[test]
fn test_screenshot_to_file() {
    let udid = get_available_udid();
    ensure_framebuffer_ready(&udid);  // Warmup

    // Screenshot test...
}
```

This ensures reliable test execution while matching real-world usage patterns.

### Related Files

- `src/cli/idb/screenshot.rs` - Screenshot command with retry logic
- `src/grpc/client.rs:252-260` - gRPC screenshot method
- `tests/screenshot_integration.rs` - Integration tests with warmup
- `tests/common/mod.rs:308-327` - `ensure_framebuffer_ready()` helper
- `proto/idb.proto:142-145` - Screenshot RPC definition

### Comparison with Python idb

| Feature | Python idb | agent-mobile (Rust) |
|---------|-----------|---------------------|
| Screenshot API | ✅ Works | ✅ Works |
| Error handling | ❌ Fails immediately | ✅ Auto-retry with backoff |
| User feedback | ❌ Generic error | ✅ Actionable suggestions |
| Test reliability | ❌ Flaky tests | ✅ Warmup helper |

**agent-mobile provides better resilience than the original Python implementation.**

---

**Last Updated**: 2026-01-18
**Status**: Mitigated with retry logic and test warmup (awaiting upstream framebuffer initialization improvement in `idb_companion`)
