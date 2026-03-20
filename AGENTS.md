# AGENTS.md

Instructions for AI coding agents working with this codebase.

## Package Manager and Workspace

This project is a Rust workspace. Use `cargo` for builds, tests, formatting, and linting.

- Main CLI binary: `agent-mobile`
- Shared crates: `crates/core`, `crates/gateway`
- Platform crates: `crates/platform-ios`, `crates/platform-android`
- iOS runner: `crates/xcuitest-runner`

Prefer workspace-aware commands unless there is a clear reason to scope to a single package.

## Code Style

- Do not use emojis in code, output, or documentation.
- CLI flags must use kebab-case. Follow the existing `clap` style such as `--session`, `--test-threads`, and `long-press`.
- When adding agent-facing query or inspection features, support machine-readable output where it fits the existing UX. Prefer the current patterns: `--json` or `-f json`.
- Error messages should state what failed and, when practical, what the user should check next.
- Keep commands idempotent where possible. Re-running the same command should not create unsafe or surprising side effects.
- Preserve the language of the file you are editing. Existing docs in this repo intentionally mix English and Japanese.

## Documentation

When adding or changing user-facing behavior such as commands, flags, output formats, session behavior, platform requirements, or examples, update all relevant documentation. At minimum, review these locations:

1. `README.md` for the primary English user guide
2. `docs/README.ja.md` for the Japanese user guide
3. Relevant deep-dive docs such as `docs/ios-runner.ja.md`, `docs/android-console.md`, and `docs/ARCHITECTURE.md`
4. The affected crate README files under `crates/`
5. `clap` help text and examples in the relevant source modules under `src/`
6. Inline doc comments near the changed code

Do not assume a feature is documented just because the main README was updated. If the change affects AI-agent workflows, platform internals, or architecture assumptions, update the corresponding focused docs too.

## Architecture

This is a Rust workspace with a thin CLI entrypoint and platform-specific backends.

- `src/main.rs` parses the top-level CLI and dispatches commands.
- `src/command.rs` defines the top-level `clap` interface.
- `src/core/` contains the main UI automation commands such as `tap`, `fill`, `find`, `wait`, `get`, `is`, `select`, `scroll`, and `swipe`.
- `src/snapshot/` handles snapshot capture, ref generation, tree printing, and cache management.
- `src/app.rs`, `src/device.rs`, `src/session/`, `src/doctor.rs`, `src/console.rs`, and `src/record.rs` implement the non-core command groups.
- `crates/platform-ios` uses `simctl`, CoreSimulator integration, and the Swift XCUITest Runner HTTP server.
- `crates/platform-android` uses ADB and UI Automator based flows for Android automation.
- `crates/gateway` provides higher-level platform resolution and shared orchestration helpers.
- `crates/xcuitest-runner` contains the Swift-side runner used by iOS automation.

Before changing command behavior, confirm which layer owns the behavior. Do not put platform-specific logic in the CLI layer if it belongs in a platform crate.

## Testing

For code changes, use this development cycle:

```bash
cargo build --workspace --verbose
cargo test --workspace --verbose
cargo test --test cli -- --test-threads=1
```

Also run the relevant quality checks when they apply:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
```

If you change a specific command, prefer running the focused integration tests in `tests/cli/` in addition to the broader test suite.

## Device Verification

Real device or simulator verification is required for command behavior changes. A successful build is not enough.

### iOS

```bash
agent-mobile <changed-command> [args...]
```

### Android

```bash
agent-mobile <changed-command> [args...]
```

After a behavior change, verify all of the following:

- The command behaves as expected on the real target
- Error messages are still appropriate
- The UI result is visually confirmed
- A screenshot or equivalent artifact is captured as evidence

Use the simulator, emulator, or real device workflow that matches the platform you changed. Document the exact verification command sequence in your work log or PR when it is not obvious from the affected feature.

Do not commit command behavior changes without real device or simulator validation.
