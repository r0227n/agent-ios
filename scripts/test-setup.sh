#!/bin/bash
# Test setup script for agent-mobile CLI integration tests
# Detects and prepares iOS or Android devices for testing

set -e

PLATFORM=${1:-auto}

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "🔧 Agent-Mobile Test Setup"
echo "=========================="
echo ""

case $PLATFORM in
  ios)
    echo "📱 Setting up iOS simulator..."

    # Check if idb is installed
    if ! command -v idb &> /dev/null; then
      echo -e "${RED}❌ Error: Python idb is not installed${NC}"
      echo "Please install idb: pip install fb-idb"
      exit 1
    fi

    # Check for booted simulators
    if xcrun simctl list devices booted | grep -q "Booted"; then
      echo -e "${GREEN}✅ iOS simulator detected${NC}"
      idb list-targets --json | head -5
      exit 0
    else
      echo -e "${YELLOW}⚠️  No booted iOS simulator found${NC}"
      echo "Please boot a simulator with:"
      echo "  xcrun simctl boot <UDID>"
      exit 1
    fi
    ;;

  android)
    echo "🤖 Setting up Android emulator..."

    # Check if adb is installed
    if ! command -v adb &> /dev/null; then
      echo -e "${RED}❌ Error: adb is not installed${NC}"
      echo "Please install Android SDK platform-tools"
      exit 1
    fi

    # Check for connected devices
    if adb devices | grep -q "device$"; then
      echo -e "${GREEN}✅ Android device/emulator detected${NC}"
      adb devices -l
      exit 0
    else
      echo -e "${YELLOW}⚠️  No Android device/emulator found${NC}"
      echo "Please start an emulator with:"
      echo "  emulator -avd <AVD_NAME>"
      echo "Or connect a physical device via USB"
      exit 1
    fi
    ;;

  auto)
    echo "🔍 Auto-detecting platform..."
    echo ""

    # Try iOS first
    if command -v idb &> /dev/null && xcrun simctl list devices booted | grep -q "Booted"; then
      echo -e "${GREEN}✅ iOS simulator detected${NC}"
      idb list-targets --json | head -5
      echo ""
      echo "Run tests with:"
      echo "  cargo test --test cli -- --test-threads=1"
      exit 0
    fi

    # Try Android
    if command -v adb &> /dev/null && adb devices | grep -q "device$"; then
      echo -e "${GREEN}✅ Android device/emulator detected${NC}"
      adb devices -l
      echo ""
      echo "Run tests with:"
      echo "  cargo test --test cli -- --test-threads=1"
      exit 0
    fi

    # No device found
    echo -e "${RED}❌ No iOS or Android device found${NC}"
    echo ""
    echo "iOS setup:"
    echo "  1. Install Python idb: pip install fb-idb"
    echo "  2. Boot a simulator: xcrun simctl boot <UDID>"
    echo ""
    echo "Android setup:"
    echo "  1. Install Android SDK platform-tools"
    echo "  2. Start emulator: emulator -avd <AVD_NAME>"
    echo "  3. Or connect a physical device"
    exit 1
    ;;

  *)
    echo -e "${RED}❌ Invalid platform: $PLATFORM${NC}"
    echo "Usage: $0 [ios|android|auto]"
    exit 1
    ;;
esac
