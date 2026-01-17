# MV Command Implementation - Final Steps

## Completed Steps ✅
1. ✅ Added `pub mod mv;` to `src/cli/idb/mod.rs`
2. ✅ Added `Mv` variant to `IdbCommands` enum in `src/cli/idb/mod.rs`
3. ✅ Created `src/cli/idb/mv.rs` with complete implementation

## Remaining Step

### Add routing to `src/main.rs`

Add the following code after the `Uninstall` match arm (around line 82):

```rust
            IdbCommands::Mv {
                src_paths,
                dst_path,
                bundle_id,
                root,
                udid,
            } => {
                cli::idb::mv::run(src_paths, dst_path, bundle_id, root, udid).await?;
            }
```

The complete context should look like:

```rust
            IdbCommands::Uninstall { bundle_id, udid } => {
                cli::idb::uninstall::run(bundle_id, udid).await?;
            }
            IdbCommands::Mv {
                src_paths,
                dst_path,
                bundle_id,
                root,
                udid,
            } => {
                cli::idb::mv::run(src_paths, dst_path, bundle_id, root, udid).await?;
            }
        },
    }
```

## After Adding Routing

Run these commands:

```bash
cd /Users/r0227n/Dev/agent-mobile-worktrees/feature-idb-file-mv

# Build the project
cargo build

# Test the implementation
./target/debug/agent-mobile idb mv --help

# Test with actual device (replace UDID)
./target/debug/agent-mobile idb mv /tmp/test1 /tmp/test2 --root --udid 1AFD3E2F-77BD-4BEC-8E3C-D9B916670F35

# Commit and push
git add .
git commit -m "feat: implement idb file mv command

This commit adds the mv command for moving files on iOS devices/simulators.

Features:
- Move single or multiple source paths to destination
- Support for root, media, and app-specific containers via --root and --bundle-id flags
- Full gRPC integration with idb_companion

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

git push -u origin feature/idb-file-mv

# Create PR
gh pr create --title "feat: implement idb file mv command" --body "$(cat <<'EOF'
## Summary
- Implements `agent-mobile idb mv` command for moving files on iOS devices/simulators
- Supports multiple source paths and various file containers (root, media, app-specific)
- Maintains compatibility with Python idb behavior

## Changes
- Added `Mv` command variant to CLI with proper argument parsing
- Implemented `src/cli/idb/mv.rs` with FileContainer support
- Added gRPC client method (already implemented in earlier commit)
- Added routing in main.rs

## Test plan
- [ ] Build succeeds without warnings
- [ ] Help text displays correctly: `./target/debug/agent-mobile idb mv --help`
- [ ] Can move files with --root flag
- [ ] Can move files with --bundle-id flag
- [ ] Can move multiple source files
- [ ] Error handling works for missing files

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

## Implementation Summary

The mv command implementation is complete with:

1. **CLI Integration** (`src/cli/idb/mod.rs`):
   - Module declaration for mv
   - Command variant with arguments: src_paths, dst_path, bundle_id, root, udid

2. **Command Implementation** (`src/cli/idb/mv.rs`):
   - Input validation (ensures at least one source path)
   - Companion connection
   - FileContainer construction based on flags:
     - `--bundle-id <ID>`: App-specific container
     - `--root`: Root file system
     - Default: Media container
   - gRPC mv call execution

3. **gRPC Integration** (`src/grpc/client.rs`):
   - Already implemented in previous commit

4. **Routing** (`src/main.rs`):
   - Needs to be added manually (see above)
