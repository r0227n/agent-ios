---
name: agent-mobile
description: Mobile automation skill for the local `agent-mobile` CLI. Use when Codex needs to operate an iOS simulator or Android device with `agent-mobile`, including checking the environment, listing or booting devices, creating sessions, launching or installing apps, capturing UI snapshots with `@eN` refs, tapping/filling/typing/selecting/scanning UI, waiting for state changes, reading element properties, taking screenshots or recordings, streaming logs, or automating end-to-end mobile app flows.
---

# agent-mobile

Use this skill to drive a mobile app through the local `agent-mobile` binary.

## Core Workflow

Every `agent-mobile` task should follow this loop:

1. Confirm the environment and target device.
2. Launch the app and capture a fresh snapshot.
3. Interact with `@eN` refs or semantic locators.
4. Wait for the next stable state.
5. Capture another snapshot or save evidence.

```bash
agent-mobile doctor
agent-mobile device list -f json
agent-mobile device boot "iPhone 15 Pro"
agent-mobile session create ios-dev --udid <UDID>
export AGENT_MOBILE_SESSION=ios-dev

agent-mobile app launch com.example.app
agent-mobile snapshot -i

agent-mobile fill @e2 "user@example.com"
agent-mobile tap @e3

agent-mobile wait text "Home" --timeout 10s
agent-mobile snapshot -i
agent-mobile screenshot -o artifacts/home.png
```

## Operating Rules

- Re-run `snapshot` after navigation, modal transitions, scrolling, picker changes, or any action that can invalidate `@eN` refs.
- Prefer `snapshot -i` for tight agent loops. Use `snapshot -f json`, `find ... -f json`, `get ... -f json`, or other JSON-capable commands when another tool must parse the result.
- Prefer `--session` or `AGENT_MOBILE_SESSION` once a device is selected. Fall back to `--udid` only for one-off commands.
- Prefer `find` when refs are not known yet, the screen is dynamic, or you want a single command with an inline action such as `tap`, `long-press`, `fill`, or `clear`.
- Use `wait` before asserting on UI changes. Use `is` when exit-code based branching is enough.
- Save proof with `screenshot` or `record` when the user asks for verification or when a flow can fail visually.

## Common Patterns

### Login Flow

```bash
agent-mobile app launch com.example.app
agent-mobile snapshot -i
agent-mobile find placeholder "Email" fill "user@example.com"
agent-mobile find placeholder "Password" fill "password123"
agent-mobile find text "Sign in" tap
agent-mobile wait text "Dashboard" --timeout 10s
agent-mobile screenshot -o artifacts/dashboard.png
```

### Query Before Acting

```bash
agent-mobile snapshot -f json
agent-mobile find type Button --all -f json
agent-mobile get text @e4 -f json
```

### Session-Based Reuse

```bash
agent-mobile session create smoke-ios --udid <UDID>
agent-mobile --session smoke-ios app launch com.example.app
agent-mobile --session smoke-ios snapshot -i
agent-mobile session show --session smoke-ios -f json
```

## Detailed Reference

Read [references/command-playbook.md](references/command-playbook.md) when you need:

- exact subcommand shapes for app, device, and session management
- platform caveats such as iOS clipboard usage or Android recording limits
- heuristics for choosing between refs, semantic locators, JSON output, and evidence capture
