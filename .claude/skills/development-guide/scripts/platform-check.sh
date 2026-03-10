#!/bin/bash
#
# platform-check.sh - iOS platform implementation decision helper
#
# Usage: ./scripts/platform-check.sh <feature-name>
#
# Recommends XCUITest Runner (HTTP) or xcrun simctl based on feature category.

set -e

# Check arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <feature-name>"
    echo ""
    echo "Example:"
    echo "  $0 accessibility"
    echo "  $0 boot"
    echo "  $0 clipboard"
    exit 1
fi

FEATURE=$(echo "$1" | tr '[:upper:]' '[:lower:]')

# XCUITest Runner (HTTP) features - UI/interaction/app operations
XCUITEST_FEATURES=(
    "tap" "swipe" "scroll" "long-press" "longpress"
    "type" "fill" "input" "text" "keyboard"
    "accessibility" "snapshot" "find" "get" "is" "wait" "element"
    "screenshot" "screen" "capture"
    "clipboard" "copy" "paste" "pasteboard"
    "app" "launch" "terminate" "activate"
    "touch" "gesture" "drag" "drop"
    "check" "uncheck" "select" "picker"
    "focus" "alert" "dialog"
    "record" "video"
)

# simctl features - device lifecycle management
SIMCTL_FEATURES=(
    "boot" "shutdown" "erase" "reset"
    "create" "delete" "clone"
    "install" "uninstall"
    "list" "device" "simulator"
    "rename" "pair" "unpair"
    "privacy" "permission"
    "status_bar" "statusbar"
    "keychain"
    "openurl" "open-url"
    "push" "notification"
    "io" "video-recording"
)

# Check if feature matches XCUITest Runner category
is_xcuitest() {
    for f in "${XCUITEST_FEATURES[@]}"; do
        if [[ "$FEATURE" == *"$f"* ]]; then
            return 0
        fi
    done
    return 1
}

# Check if feature matches simctl category
is_simctl() {
    for f in "${SIMCTL_FEATURES[@]}"; do
        if [[ "$FEATURE" == *"$f"* ]]; then
            return 0
        fi
    done
    return 1
}

echo "Checking implementation recommendation for: $FEATURE"
echo ""

XCUITEST_MATCH=false
SIMCTL_MATCH=false

if is_xcuitest; then
    XCUITEST_MATCH=true
fi

if is_simctl; then
    SIMCTL_MATCH=true
fi

if $XCUITEST_MATCH && $SIMCTL_MATCH; then
    echo "⚡ Hybrid: Both XCUITest Runner and simctl match"
    echo ""
    echo "Recommendation: Hybrid implementation"
    echo "  - Primary: XCUITest Runner (HTTP) via with_xcuitest()"
    echo "  - Fallback: xcrun simctl for lifecycle/permissions"
    echo ""
    echo "Implementation:"
    echo "  pub async fn run() -> CommandResult {"
    echo "      // Try XCUITest Runner first"
    echo "      match with_xcuitest(|client| async move {"
    echo "          client.<method>().await"
    echo "      }).await {"
    echo "          Ok(result) => Ok(result),"
    echo "          Err(_) => {"
    echo "              // Fallback to simctl"
    echo "              simctl::management::<method>(udid).await"
    echo "          }"
    echo "      }"
    echo "  }"
    echo ""
elif $XCUITEST_MATCH; then
    echo "✓ Recommendation: Use XCUITest Runner (HTTP)"
    echo ""
    echo "Implementation:"
    echo "  - Use with_xcuitest() pattern (no UDID parameter)"
    echo "  - Location: src/core/<feature>.rs"
    echo ""
    echo "Example:"
    echo "  async fn run_ios() -> CommandResult {"
    echo "      with_xcuitest(|client| async move {"
    echo "          client.<method>().await?;"
    echo "          Ok(())"
    echo "      }).await"
    echo "  }"
    echo ""
    echo "References:"
    echo "  - src/helpers/client.rs (with_xcuitest implementation)"
    echo "  - crates/platform-ios/src/xcuitest/client.rs (XCUITestClient API)"
elif $SIMCTL_MATCH; then
    echo "✓ Recommendation: Use xcrun simctl"
    echo ""
    echo "Implementation:"
    echo "  - Use simctl::management module"
    echo "  - Location: crates/platform-ios/src/simctl/management.rs"
    echo ""
    echo "Common simctl commands:"
    echo "  - boot <udid>              # Start simulator"
    echo "  - shutdown <udid>          # Stop simulator"
    echo "  - create <name> <type>     # Create simulator"
    echo "  - delete <udid>            # Delete simulator"
    echo "  - install <udid> <app>     # Install app"
    echo "  - uninstall <udid> <id>    # Uninstall app"
    echo ""
    echo "References:"
    echo "  - crates/platform-ios/src/simctl/management.rs (simctl wrapper)"
    echo "  - crates/platform-ios/src/simctl/cache.rs (device cache, TTL 5s)"
else
    echo "? No matching category found for: $FEATURE"
    echo ""
    echo "Manual decision needed. Consider:"
    echo ""
    echo "  Use XCUITest Runner (HTTP) if:"
    echo "    - Feature involves UI interaction"
    echo "    - Feature reads screen/element state"
    echo "    - Feature operates on running app"
    echo ""
    echo "  Use xcrun simctl if:"
    echo "    - Feature manages simulator lifecycle"
    echo "    - Feature is device-level operation"
    echo "    - Feature doesn't need app context"
fi

echo ""
echo "Decision criteria:"
echo "  ✓ UI/element operation  → XCUITest Runner (HTTP, localhost:8200)"
echo "  ✓ App interaction       → XCUITest Runner (HTTP)"
echo "  ✓ Screenshot/capture    → XCUITest Runner (HTTP)"
echo "  ✓ Clipboard             → XCUITest Runner (HTTP, UIPasteboard)"
echo "  ✓ Simulator lifecycle   → xcrun simctl"
echo "  ✓ Device management     → xcrun simctl"
echo "  ✓ Device detection      → simctl::list_simulators() (cached, TTL 5s)"
