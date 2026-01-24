#!/usr/bin/env bash
set -euo pipefail

# Android Device Setup Script for agent-mobile

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

echo "=== Android Device Setup ==="
echo ""

# Step 1: Check Android SDK
log_section "Checking Android SDK..."

# Check adb
adb_path=$(command -v adb 2>/dev/null || true)
if [ -n "$adb_path" ]; then
  log_ok "adb found at $adb_path"
else
  log_fail "adb not found"
  echo ""
  echo "Please add Android SDK to PATH:"
  echo "  export ANDROID_HOME=\$HOME/Library/Android/sdk"
  echo "  export PATH=\$PATH:\$ANDROID_HOME/platform-tools"
  exit 1
fi

# Check emulator
emulator_path=$(command -v emulator 2>/dev/null || true)
if [ -n "$emulator_path" ]; then
  log_ok "emulator found at $emulator_path"
else
  log_fail "emulator not found"
  echo ""
  echo "Please add Android SDK emulator to PATH:"
  echo "  export PATH=\$PATH:\$ANDROID_HOME/emulator"
  exit 1
fi

# Step 2: Check adb server
log_section "Checking adb server..."

if adb get-state &>/dev/null || adb start-server &>/dev/null; then
  log_ok "adb server running"
else
  log_info "Starting adb server..."
  adb start-server
  sleep 1
  if adb get-state &>/dev/null; then
    log_ok "adb server started"
  else
    log_fail "Could not start adb server"
  fi
fi

# Step 3: Check emulator status
log_section "Checking emulator status..."

# List available AVDs
avd_list=$(emulator -list-avds 2>/dev/null || true)
if [ -n "$avd_list" ]; then
  log_info "Available AVDs:"
  echo "$avd_list" | while read -r avd; do
    echo "    - $avd"
  done
else
  log_fail "No AVDs available"
  echo ""
  echo "Please create an AVD using Android Studio or avdmanager"
  exit 1
fi

# Check for connected devices
connected_devices=$(adb devices 2>/dev/null | grep -E "emulator-|device$" | head -1 || true)

if [ -n "$connected_devices" ]; then
  device_id=$(echo "$connected_devices" | awk '{print $1}')
  log_ok "Emulator already running: $device_id"
else
  log_info "No running emulator found"

  # Get first available AVD
  target_avd=$(echo "$avd_list" | head -1)

  if [ -n "$target_avd" ]; then
    log_section "Booting emulator..."
    log_info "Starting $target_avd..."

    # Start emulator in background
    nohup emulator -avd "$target_avd" -no-snapshot-load >/dev/null 2>&1 &
    emulator_pid=$!

    log_info "Emulator process started (PID: $emulator_pid)"
    log_info "Waiting for boot (this may take a while)..."

    # Wait for device
    max_wait=120
    waited=0
    while [ $waited -lt $max_wait ]; do
      if adb wait-for-device shell getprop sys.boot_completed 2>/dev/null | grep -q "1"; then
        log_ok "Emulator booted successfully"
        device_id=$(adb devices | grep -E "emulator-" | head -1 | awk '{print $1}')
        break
      fi
      sleep 5
      waited=$((waited + 5))
      echo -n "."
    done
    echo ""

    if [ $waited -ge $max_wait ]; then
      log_fail "Emulator boot timeout"
      echo ""
      echo "Try starting emulator manually:"
      echo "  emulator -avd $target_avd"
      exit 1
    fi
  else
    log_fail "No AVD to boot"
    exit 1
  fi
fi

# Step 4: Verify connection
log_section "Verifying connection..."

# Wait a moment for everything to settle
sleep 2

if agent-mobile device list 2>/dev/null | grep -qi "android\|emulator"; then
  log_ok "agent-mobile can communicate with device"
else
  log_info "Checking with adb..."
  if adb devices | grep -q "device$"; then
    log_ok "adb can communicate with device"
    log_info "agent-mobile may need Android support enabled"
  else
    log_fail "Cannot communicate with device"
  fi
fi

# Summary
echo ""
echo "=== Setup Complete ==="
if [ -n "${device_id:-}" ]; then
  echo "Device: $device_id"
fi
echo "Status: Ready for testing"
echo ""
echo "Next steps:"
echo "  agent-mobile device list    # Verify device"
echo "  adb shell                   # Direct shell access"
echo "  /mobile-e2e android         # Run E2E tests"
