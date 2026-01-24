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

# Step 2: Check idb_companion
log_section "Checking idb_companion..."

idb_path=$(command -v idb_companion 2>/dev/null || true)
if [ -n "$idb_path" ]; then
  log_ok "idb_companion found at $idb_path"
else
  log_fail "idb_companion not found"
  echo ""
  echo "Please install idb_companion:"
  echo "  brew install idb-companion"
  exit 1
fi

# Check for running companions
companion_count=$(pgrep -c idb_companion 2>/dev/null || echo "0")
if [ "$companion_count" -gt 0 ]; then
  log_info "Found $companion_count running idb_companion process(es)"
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

# Wait a moment for companion to spawn
sleep 2

if agent-mobile device list 2>/dev/null | grep -qi "ios\|iphone\|ipad"; then
  log_ok "agent-mobile can communicate with device"
else
  log_info "Waiting for idb_companion to connect..."
  sleep 3

  if agent-mobile device list 2>/dev/null | grep -qi "ios\|iphone\|ipad"; then
    log_ok "agent-mobile can communicate with device"
  else
    log_fail "agent-mobile cannot communicate with device"
    echo ""
    echo "Try manually checking with:"
    echo "  agent-mobile device list"
  fi
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
