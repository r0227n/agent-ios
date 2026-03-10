#!/usr/bin/env bash
set -euo pipefail

# iOS Device Setup Script for agent-mobile

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_ok() {
  echo -e "  ${GREEN}[OK]${NC} $1"
}

log_fail() {
  echo -e "  ${RED}[FAIL]${NC} $1"
}

log_info() {
  echo -e "  ${BLUE}[INFO]${NC} $1"
}

log_section() {
  echo -e "${YELLOW}>>>${NC} $1"
}

echo "=== iOS Device Setup ==="
echo ""

# Step 1: Check Xcode installation
log_section "Checking Xcode installation..."

if command -v xcrun &>/dev/null && xcrun simctl list devices &>/dev/null; then
  log_ok "xcrun simctl available"
else
  log_fail "xcrun simctl not available"
  echo ""
  echo "Please install Xcode Command Line Tools:"
  echo "  xcode-select --install"
  exit 1
fi

# Step 2: Check XCUITest Runner
log_section "Checking XCUITest Runner..."

XCUITEST_RUNNER_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)/crates/xcuitest-runner"
if [ -d "$XCUITEST_RUNNER_DIR" ]; then
  log_ok "XCUITest Runner project found"
else
  log_fail "XCUITest Runner project not found at $XCUITEST_RUNNER_DIR"
  echo ""
  echo "XCUITest Runner is required for iOS automation."
  echo "Ensure crates/xcuitest-runner/ exists in the project."
  exit 1
fi

# Step 3: Check simulator status
log_section "Checking simulator status..."

# Get list of available simulators
available_sims=$(xcrun simctl list devices available -j 2>/dev/null | grep -o '"name" : "[^"]*"' | head -5 || true)

# Check for booted simulators
booted_sim=$(xcrun simctl list devices | grep -E "Booted" | head -1 || true)

if [ -n "$booted_sim" ]; then
  log_ok "Simulator already booted"
  log_info "$booted_sim"

  # Extract UDID
  udid=$(echo "$booted_sim" | grep -oE '[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}' || true)
else
  log_info "No booted simulator found"

  # Find a suitable simulator to boot
  # Prefer iPhone 15 or latest iPhone
  target_sim=""
  target_udid=""

  while IFS= read -r line; do
    if [[ "$line" =~ iPhone\ 1[5-9] ]] || [[ "$line" =~ iPhone\ [2-9][0-9] ]]; then
      sim_name=$(echo "$line" | sed 's/^[[:space:]]*//' | cut -d'(' -f1 | sed 's/[[:space:]]*$//')
      sim_udid=$(echo "$line" | grep -oE '[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}' || true)
      if [ -n "$sim_udid" ]; then
        target_sim="$sim_name"
        target_udid="$sim_udid"
        break
      fi
    fi
  done < <(xcrun simctl list devices available | grep -i "iphone")

  # Fallback to any available iPhone
  if [ -z "$target_udid" ]; then
    while IFS= read -r line; do
      if [[ "$line" =~ iPhone ]]; then
        sim_name=$(echo "$line" | sed 's/^[[:space:]]*//' | cut -d'(' -f1 | sed 's/[[:space:]]*$//')
        sim_udid=$(echo "$line" | grep -oE '[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}' || true)
        if [ -n "$sim_udid" ]; then
          target_sim="$sim_name"
          target_udid="$sim_udid"
          break
        fi
      fi
    done < <(xcrun simctl list devices available | grep -i "iphone")
  fi

  if [ -n "$target_udid" ]; then
    log_section "Booting simulator..."
    log_info "Starting $target_sim..."

    if xcrun simctl boot "$target_udid" 2>/dev/null; then
      log_ok "Simulator booted successfully"
      udid="$target_udid"

      # Open Simulator app
      open -a Simulator 2>/dev/null || true

      # Wait for boot to complete
      sleep 3
    else
      # May already be booted or booting
      udid="$target_udid"
      log_info "Simulator may already be starting"
    fi
  else
    log_fail "No available iPhone simulator found"
    echo ""
    echo "Please create a simulator using Xcode or:"
    echo "  xcrun simctl create \"iPhone 15\" \"iPhone 15\" \"iOS17.0\""
    exit 1
  fi
fi

# Step 4: Verify connection
log_section "Verifying connection..."

# Wait for simulator to settle
sleep 2

if agent-mobile device list 2>/dev/null | grep -qi "ios\|iphone\|ipad"; then
  log_ok "agent-mobile can detect device"
else
  log_info "Waiting for simulator to be ready..."
  sleep 3

  if agent-mobile device list 2>/dev/null | grep -qi "ios\|iphone\|ipad"; then
    log_ok "agent-mobile can detect device"
  else
    log_fail "agent-mobile cannot detect device"
    echo ""
    echo "Try manually checking with:"
    echo "  agent-mobile device list"
    echo "  xcrun simctl list devices | grep Booted"
  fi
fi

# Step 5: Check XCUITest Runner health
log_section "Checking XCUITest Runner health..."

if curl -s --max-time 5 http://localhost:8200/health &>/dev/null; then
  log_ok "XCUITest Runner is responding (localhost:8200)"
else
  log_info "XCUITest Runner not running yet (will auto-start on first command)"
  log_info "Run 'agent-mobile snapshot' to trigger auto-start"
fi

# Summary
echo ""
echo "=== Setup Complete ==="
if [ -n "${udid:-}" ]; then
  echo "UDID: $udid"
fi
echo "Status: Ready for testing"
echo ""
echo "Next steps:"
echo "  agent-mobile device list    # Verify device"
echo "  agent-mobile snapshot       # Get UI snapshot"
echo "  /mobile-e2e ios             # Run E2E tests"
