# File Command Integration Tests

This document describes the integration tests for `agent-mobile idb file` commands, which verify compatibility with Python's `idb file` commands.

## Overview

The integration tests compare the behavior of Python `idb` and Rust `agent-mobile` implementations to ensure identical functionality.

### Command Structure

- **Python idb**: `idb file <subcommand> [args]`
- **agent-mobile**: `agent-mobile idb file <subcommand> [args]`

Both should produce identical outputs for the same inputs.

## Test Files

All integration tests are located in the `tests/` directory:

- **tests/common/mod.rs** - Shared test utilities
- **tests/file_ls_integration.rs** - Tests for `ls` command
- **tests/file_mkdir_integration.rs** (mkdir_integration.rs) - Tests for `mkdir` command
- **tests/file_mv_integration.rs** - Tests for `mv` command
- **tests/file_rm_integration.rs** - Tests for `rm` command
- **tests/file_pull_integration.rs** - Tests for `pull` command
- **tests/file_push_integration.rs** - Tests for `push` command
- **tests/file_tail_integration.rs** - Tests for `tail` command

## Test Utilities (tests/common/mod.rs)

### Key Functions

- `get_available_udid()` - Gets UDID of first booted simulator
- `ensure_companion_running(udid)` - Ensures idb-companion is running
- `run_idb_file_command(args)` - Executes Python `idb file` command
- `run_agent_mobile_file_command(args)` - Executes `agent-mobile idb file` command
- `compare_file_command_outputs(python, rust)` - Compares outputs for equivalence
- `build_agent_mobile()` - Builds the Rust binary

## Test Coverage

### 1. ls (List Files) - 6 tests

- Basic directory listing equivalence
- Multiple paths listing
- List with bundle ID
- Non-existent path error handling
- List without UDID (auto-detect)
- Root directory listing

### 2. mkdir (Make Directory) - 6 tests

- Basic directory creation
- Directory creation without UDID
- Invalid bundle ID error handling
- Compatibility with Python idb
- Directory creation with bundle ID
- Nested directory creation

### 3. mv (Move Files) - 5 tests

- Basic file move equivalence
- Multiple files move
- Move with --root flag
- Source not found error handling
- File rename equivalence

### 4. rm (Remove Files) - 6 tests

- Single file removal
- Multiple files removal
- Directory recursive removal
- Non-existent file error handling
- Remove with bundle ID
- Empty directory removal

### 5. pull (Pull from Device) - 5 tests

- Pull file to local path
- Pull to stdout (-)
- Pull with bundle ID
- Non-existent file error handling
- Binary file pull equivalence

### 6. push (Push to Device) - 5 tests

- Push file to device
- Push with bundle ID
- Binary file push equivalence
- Non-existent source error handling
- Overwrite existing file

### 7. tail (Tail File) - 5 tests

- Tail existing file with SIGTERM handling
- Tail with bundle ID
- Non-existent file error handling
- Empty file tail equivalence
- Signal handling (SIGTERM)

**Total: 38 integration tests**

## Running Tests

### Prerequisites

1. **Install Python idb**:
   ```bash
   pip install fb-idb
   ```

2. **Install idb-companion**:
   ```bash
   brew tap facebook/fb
   brew install idb-companion
   ```

3. **Boot a simulator**:
   ```bash
   xcrun simctl boot <UDID>
   # Or boot via Xcode
   ```

### Running All Integration Tests

```bash
# Build the project first
cargo build

# Run all integration tests (single-threaded to avoid conflicts)
cargo test --tests -- --ignored --test-threads=1
```

### Running Specific Test Files

```bash
# Test ls command
cargo test --test file_ls_integration -- --ignored

# Test mkdir command
cargo test --test mkdir_integration -- --ignored

# Test mv command
cargo test --test file_mv_integration -- --ignored

# Test rm command
cargo test --test file_rm_integration -- --ignored

# Test pull command
cargo test --test file_pull_integration -- --ignored

# Test push command
cargo test --test file_push_integration -- --ignored

# Test tail command
cargo test --test file_tail_integration -- --ignored
```

### Running Specific Tests

```bash
# Run a specific test function
cargo test --test file_ls_integration test_ls_basic_equivalence -- --ignored
```

## Test Patterns

### Standard Test Pattern

```rust
#[test]
#[ignore] // Run with: cargo test --test <file> -- --ignored
fn test_command_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    // Run Python idb command
    let python_output = common::run_idb_file_command(&[
        "<subcommand>", "<args>", "--udid", &udid
    ]);

    // Run agent-mobile command
    let rust_output = common::run_agent_mobile_file_command(&[
        "<subcommand>", "<args>", "--udid", &udid
    ]);

    // Compare outputs
    common::compare_file_command_outputs(&python_output, &rust_output);
}
```

### Error Test Pattern

```rust
#[test]
#[ignore]
fn test_command_error_handling() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let python_output = common::run_idb_file_command(&[...]);
    let rust_output = common::run_agent_mobile_file_command(&[...]);

    // Both should fail
    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    common::compare_file_command_outputs(&python_output, &rust_output);
}
```

## Output Comparison

The `compare_file_command_outputs` function compares:

1. **Exit Codes** - Must match exactly
2. **stdout** - Trimmed and compared for exact match
3. **stderr** - Relaxed comparison (both succeed or both fail)

### Normalization

- Trailing whitespace is trimmed
- Both implementations must have same exit code
- Error messages may differ slightly, but error states must match

## CI Integration

The GitHub Actions workflow (`.github/workflows/idb-ci.yml`) automatically runs these tests:

```yaml
- name: Install Python idb CLI
  run: pip install fb-idb

- name: Setup test environment
  run: mise run test-setup

- name: Run integration tests
  run: cargo test --tests -- --test-threads=1
```

Tests run on `macos-15-intel` runners with:
- Python idb installed
- idb-companion installed
- Booted simulator available

## Success Criteria

All integration tests must pass to confirm that `agent-mobile idb file` commands produce identical output to Python `idb file` commands.

### What "Identical Output" Means

- Same exit code (0 for success, non-zero for errors)
- Same stdout content (after normalization)
- Same error behavior (both succeed or both fail)
- Same file operations (verified by pull/push roundtrips)

## Troubleshooting

### No Booted Simulator

```
Error: No booted simulator available
```

**Solution**: Boot a simulator first
```bash
xcrun simctl boot <UDID>
```

### Python idb Not Found

```
Error: Failed to execute Python idb - ensure idb is installed
```

**Solution**: Install Python idb
```bash
pip install fb-idb
```

### Companion Not Running

The tests automatically start the companion via `ensure_companion_running()`, but if issues persist:

```bash
# Check companion status
idb describe --udid <UDID>
```

### Test Failures

If a test fails:

1. Run the specific test with verbose output:
   ```bash
   cargo test --test file_ls_integration test_ls_basic -- --ignored --nocapture
   ```

2. Compare command outputs manually:
   ```bash
   # Python
   idb file ls /tmp --udid <UDID>

   # Rust
   ./target/debug/agent-mobile idb file ls /tmp --udid <UDID>
   ```

3. Check for differences in output format, exit codes, or error messages

## Implementation Status

✅ All 7 file commands implemented:
- `ls` - List files
- `mkdir` - Create directories
- `mv` - Move/rename files
- `rm` - Remove files/directories
- `pull` - Pull files from device
- `push` - Push files to device
- `tail` - Tail file with streaming

✅ Comprehensive test coverage (38 tests)
✅ CI integration configured
✅ All tests passing locally

## Next Steps

1. Ensure all tests pass in CI
2. Create pull request to merge `fix/idb-file` → `main`
3. Verify tests pass in PR checks
4. Merge when all checks pass

## References

- Python idb documentation: https://fbidb.io/
- Project README: `README.md`
- CI workflow: `.github/workflows/idb-ci.yml`
- Implementation plan: See original task description
