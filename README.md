# agent-mobile

> AI-friendly mobile automation CLI for iOS simulators and Android devices

[日本語](docs/README.ja.md)

## Additional docs

- Web documentation: https://r0227n.github.io/agent-mobile/
- [iOS Runner documentation (Japanese)](docs/ios-runner.ja.md)

`agent-mobile` is a Rust CLI for agentic mobile testing and automation. It captures the current UI, assigns short element references such as `@e1`, and lets you drive apps with concise commands like `tap`, `fill`, `find`, `wait`, and `screenshot`.

> [!NOTE]
> This project is inspired by [agent-browser](https://github.com/vercel-labs/agent-browser).

> [!IMPORTANT]
> `agent-mobile` is under active development. Commands, behavior, and APIs may change as the project evolves.

## Why agent-mobile?

- **Compact for LLM loops**: `tap @e3` is cheaper and clearer than long selector expressions.
- **Semantic locators**: Find elements by `text`, `label`, `placeholder`, `type`, or state.
- **Cross-platform workflow**: Use one CLI across iOS and Android.
- **Session-aware**: Bind a device to a named session for repeatable multi-device workflows.
- **Machine-readable output**: Query-style commands support JSON output.

## What it can do

- Capture the UI tree and generate element references
- Tap, long-press, type, fill, scroll, and swipe
- Launch, install, terminate, and uninstall apps
- Wait for elements or text to appear or disappear
- Save screenshots, record video, and stream console logs
- Manage devices and named sessions

## Installation

### Prerequisites

- Rust toolchain
- macOS + Xcode Command Line Tools for iOS simulator support
- Android SDK Platform Tools (`adb`) for Android support

### Build from source

```bash
git clone https://github.com/r0227n/agent-mobile.git
cd agent-mobile

cargo build --release
cargo install --path .

agent-mobile doctor
agent-mobile --help
```

### Add Cargo bin to PATH

If `agent-mobile` is not found after `cargo install`, add Cargo's bin directory to your `PATH`.

Check where Cargo installs binaries:

```bash
cargo install --path .
cargo install --list | rg '^agent-mobile '
```

Most environments use one of these locations:

- `$CARGO_HOME/bin` (if `CARGO_HOME` is set)
- `$HOME/.cargo/bin` (default)

Current shell only:

```bash
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
agent-mobile --help
```

Persist it in your shell config:

- zsh: add to `~/.zshrc`
- bash: add to `~/.bashrc` or `~/.bash_profile`
- fish: run `fish_add_path (string join / (or $CARGO_HOME $HOME/.cargo) bin)`

Then open a new terminal (or reload your shell config) and verify:

```bash
command -v agent-mobile
agent-mobile --help
```

### Platform notes

**iOS**

```bash
xcode-select --install
```

`agent-mobile` uses `xcrun simctl` and the bundled XCUITest Runner to automate iOS simulators.

**Android**

Install Android Studio or the Android SDK Platform Tools, then make sure `adb` is on `PATH`:

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export PATH="$PATH:$ANDROID_HOME/platform-tools"
adb version
```

## Quick start

### 1. Check your environment

```bash
agent-mobile doctor
```

### 2. Pick a device

```bash
agent-mobile device list
agent-mobile device boot "iPhone 15 Pro"
```

### 3. Launch an app

```bash
agent-mobile app launch com.apple.mobilesafari --fresh
```

### 4. Capture the current UI

```bash
agent-mobile snapshot
```

Example output:

```text
@e1  Button     "Continue"                     [enabled]
@e2  TextField  "Search or enter website name" [enabled]
@e3  Button     "Cancel"                       [enabled]
```

### 5. Interact with the UI

```bash
agent-mobile tap @e1
agent-mobile tap center --duration 0.5
agent-mobile fill @e2 "https://example.com"
```

Or use semantic locators directly:

```bash
agent-mobile find text "Continue" tap
agent-mobile find type Button --udid <DEVICE_UDID>
agent-mobile find placeholder "Search or enter website name" fill "https://example.com"
```

### 6. Verify the result

```bash
agent-mobile wait text "Example Domain" --timeout 10s
agent-mobile screenshot --output result.png
```

## Core concepts

### 1. Snapshots and element references

`snapshot` captures the accessibility tree and assigns short references such as `@e1`, `@e2`, and `@e3`.

```bash
agent-mobile snapshot
agent-mobile tap @e1
agent-mobile get @e2 text
```

Use references when you want the shortest possible interaction loop.

### 2. Semantic locators

You can also search by meaning instead of by generated refs.

```bash
agent-mobile find text "Login"
agent-mobile find label "Email" fill "user@example.com"
agent-mobile find type Button --nth 0 tap
agent-mobile find enabled --all -f json
```

Supported locator families:

- `type`
- `text`
- `label`
- `placeholder`
- `enabled`
- `disabled`

Supported inline actions:

- `tap`
- `long-press`
- `fill`
- `clear`

### 3. Sessions

Sessions let you bind a device UDID to a name and reuse it across commands.

```bash
agent-mobile session create ios-dev --udid <UDID>
agent-mobile --session ios-dev snapshot

export AGENT_MOBILE_SESSION=ios-dev
agent-mobile session show
```

### 4. JSON output

Query-oriented commands expose machine-readable output with command-specific flags such as `-f json`.

```bash
agent-mobile snapshot -f json
agent-mobile device list -f json
agent-mobile session list -f json
agent-mobile doctor --json
```

## Command overview

| Area | Commands |
| --- | --- |
| Interaction | `tap`, `long-press`, `fill`, `type`, `check`, `uncheck`, `select`, `scroll`, `swipe` |
| UI inspection | `snapshot`, `find`, `get`, `is`, `wait` |
| Media and logs | `screenshot`, `record`, `console` |
| App management | `app launch`, `app terminate`, `app install`, `app uninstall`, `app list`, `app grant`, `app revoke`, `app reset` |
| Device management | `device list`, `device boot`, `device shutdown`, `device pbcopy`, `device pbpaste` |
| Session management | `session create`, `session list`, `session show`, `session rm` |
| Environment checks | `doctor` |

Run `agent-mobile <command> --help` for command-specific details.

## Example: login flow

```bash
agent-mobile app launch com.example.app --fresh
agent-mobile snapshot

agent-mobile fill @e1 "user@example.com"
agent-mobile fill @e2 "password123"
agent-mobile tap @e3

agent-mobile wait text "Profile" --timeout 5s
agent-mobile screenshot --output logged_in.png
```

## Architecture at a glance

- `agent-mobile` is the Rust CLI entrypoint.
- `crates/core` contains shared types and traits.
- `crates/platform-ios` handles iOS automation via `simctl` and XCUITest Runner.
- `crates/platform-android` handles Android automation via `adb` and UI Automator.
- `crates/gateway` provides higher-level platform resolution and orchestration.
- `crates/xcuitest-runner` contains the Swift-side HTTP server used by iOS automation.

## Documentation

- [docs/README.ja.md](docs/README.ja.md) for the Japanese reference
- [docs/README.md](docs/README.md) for technical notes
- [CLAUDE.md](CLAUDE.md) for the development workflow
- [AGENTS.md](AGENTS.md) for repository-specific agent guidance

## Contributing

Contributions are welcome. If you change CLI behavior, update the docs and verify the flow on a real simulator or device.

## License

MIT. See [LICENSE](LICENSE).
