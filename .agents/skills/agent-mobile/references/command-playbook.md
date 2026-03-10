# agent-mobile command playbook

## Preflight

Start here when the active device or toolchain is unclear.

```bash
agent-mobile doctor
agent-mobile device list -f json
agent-mobile device boot "iPhone 15 Pro"
agent-mobile device boot "Pixel_8_API_34" --headless
```

- Use `doctor --json` when another tool needs structured diagnostics.
- Use `device list -p ios` or `device list -p android` to narrow discovery.
- Use `device shutdown --udid <UDID>` for explicit teardown.

## Sessions

Prefer sessions for multi-step work so later commands do not need `--udid`.

```bash
agent-mobile session create ios-dev --udid <UDID>
agent-mobile --session ios-dev snapshot -i
agent-mobile session list -f json
agent-mobile session show --session ios-dev -f json
agent-mobile session destroy ios-dev
```

## App lifecycle

```bash
agent-mobile app launch com.example.app -f json
agent-mobile app terminate com.example.app
agent-mobile app install ./build/MyApp.app -f json
agent-mobile app uninstall com.example.app
agent-mobile app list -f json
agent-mobile app grant camera --bundle com.example.app
agent-mobile app revoke location --bundle com.example.app
agent-mobile app reset notifications --bundle com.example.app
```

- `launch`, `install`, and `list` support JSON output.
- Bundle identifiers and Android package names share the same argument slot.

## Snapshot and interaction loop

Use the same loop as `agent-browser`: snapshot, interact, wait, snapshot again.

```bash
agent-mobile snapshot -i
agent-mobile tap @e1
agent-mobile wait visible @e4 --timeout 5s
agent-mobile snapshot -i
```

- Treat `@eN` refs as snapshot-local. Re-snapshot after UI changes.
- Use `snapshot -c` to remove empty structural nodes.
- Use `snapshot -d 3` to limit depth.
- Use `snapshot -s @e5` or `snapshot -s "Settings"` to scope to a subtree.
- Use `snapshot --no-scroll` when automatic scroll stitching is harmful.

## Semantic search

Use `find` before refs exist or when a one-shot command is cleaner.

```bash
agent-mobile find text "Continue" tap
agent-mobile find placeholder "Email" fill "user@example.com"
agent-mobile find label "Search" --exact
agent-mobile find type Button --nth 1
agent-mobile find enabled --all -f json
```

- Locator families: `type`, `text`, `label`, `placeholder`, `enabled`, `disabled`
- Match controls: `--first`, `--last`, `--nth`, `--all`
- `text`, `label`, and `placeholder` support `--exact`
- Inline actions accept `tap`, `long-press`, `fill`, or `clear`

## Direct interaction

```bash
agent-mobile tap @e3
agent-mobile long-press @e3 --duration 1.5
agent-mobile fill @e2 "hello@example.com"
agent-mobile type " more text"
agent-mobile check "Enable notifications" -f json
agent-mobile uncheck "Remember me" -f json
agent-mobile select @e7 "Tokyo"
agent-mobile swipe up --distance 500
agent-mobile swipe 100,500,100,120
agent-mobile scroll down --in @e9 --distance 400
```

- `type` appends to the focused field. `fill` clears first.
- `check` and `uncheck` are idempotent.
- `swipe` accepts either a direction or explicit coordinates.
- `scroll` can target a container with `--in`.

## Reading state

```bash
agent-mobile get text @e4
agent-mobile get value @e2 -f json
agent-mobile get attr @e4 enabled -f json
agent-mobile get box @e4 -f json
agent-mobile is visible @e4
agent-mobile is checked "Remember me"
agent-mobile wait text "Saved" --timeout 10s
agent-mobile wait idle
```

- `is` checks `visible`, `exists`, `enabled`, `disabled`, `interactive`, or `checked`.
- `wait` conditions are `visible`, `gone`, `idle`, and `text`.
- `wait idle` does not require a target.

## Evidence and logs

```bash
agent-mobile screenshot -o artifacts/result.png
agent-mobile screenshot -o artifacts/ -f jpeg
agent-mobile record -o artifacts/flow.mp4 -t 5
agent-mobile console -o artifacts/device.log
```

- `screenshot` writes to a file or directory and infers mode from the output path.
- `record` writes MP4 output. On Android, the underlying recorder is capped at 180 seconds.
- Use `console` when the user needs runtime logs alongside UI evidence.

## Clipboard and platform caveats

```bash
agent-mobile device pbcopy "hello"
agent-mobile device pbpaste
```

- Clipboard helpers are primarily for iOS simulator workflows.
- If device commands fail, rerun `doctor` and confirm the target with `device list`.
- If a locator is ambiguous, refine it with `--exact` or index selection.
