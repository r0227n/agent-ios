#!/bin/bash
#
# run-tests.sh - Guide for running 3-layer tests
#
# Usage: ./scripts/run-tests.sh [test-name]
#
# Provides step-by-step test execution guidance.

set -e

TEST_NAME="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo " agent-mobile Test Execution Guide"
echo "=========================================="
echo ""

if [ -n "$TEST_NAME" ]; then
    echo "Testing: $TEST_NAME"
    echo ""
fi

echo "Step 0: Environment Check"
echo "-------------------------"
echo "Description: Verify development environment is ready"
echo ""
echo "If devices are not detected, run:"
echo "  iOS:     ${SCRIPT_DIR}/setup-ios.sh"
echo "  Android: ${SCRIPT_DIR}/setup-android.sh"
echo ""
echo "Checking device availability..."
if ! command -v agent-mobile &>/dev/null; then
    echo "⚠️  agent-mobile not found. Build first: cargo build"
else
    agent-mobile device list 2>/dev/null || echo "⚠️  No devices detected. Run setup scripts."
fi
echo ""

echo "Step 1: Unit Tests"
echo "-------------------"
echo "Description: Tests internal logic without external dependencies"
echo "Location: #[cfg(test)] mod tests within src/ files"
echo ""
if [ -n "$TEST_NAME" ]; then
    echo "Command:"
    echo "  cargo test --verbose --lib $TEST_NAME"
else
    echo "Command:"
    echo "  cargo test --verbose --bins"
fi
echo ""

echo "Step 2: Integration Tests"
echo "-------------------------"
echo "Description: Tests CLI commands with real devices/simulators"
echo "Location: tests/cli/<name>_integration.rs"
echo ""
echo "Prerequisites:"
echo "  - iOS: Simulator running with XCUITest Runner (auto-starts on first command)"
echo "  - Android: Emulator running with adb server"
echo ""
if [ -n "$TEST_NAME" ]; then
    echo "Command:"
    echo "  cargo test --test cli $TEST_NAME -- --test-threads=1"
else
    echo "Command:"
    echo "  cargo test --test cli -- --test-threads=1"
fi
echo ""

echo "Step 3: Real Device Verification (REQUIRED!)"
echo "---------------------------------------------"
echo "Description: Manual verification on actual device/simulator"
echo "Purpose: Ensure visual behavior matches expectations"
echo ""
echo "⚠️  CRITICAL: Do NOT commit without completing this step!"
echo "    Build success ≠ Correct behavior"
echo ""
echo "iOS Verification:"
echo "  1. Start test environment:"
echo "     $ /mobile-e2e ios"
echo ""
echo "  2. Run command:"
if [ -n "$TEST_NAME" ]; then
    echo "     $ agent-mobile $TEST_NAME [args]"
else
    echo "     $ agent-mobile <command> [args]"
fi
echo ""
echo "  3. Visually verify result in simulator"
echo ""
echo "  4. Save evidence:"
echo "     $ agent-mobile screenshot /tmp/evidence.png"
echo ""
echo "  5. Test error cases:"
if [ -n "$TEST_NAME" ]; then
    echo "     $ agent-mobile $TEST_NAME --udid invalid-udid"
else
    echo "     $ agent-mobile <command> --udid invalid-udid"
fi
echo "     → Verify error message is clear and actionable"
echo ""
echo "Android Verification:"
echo "  1. Start test environment:"
echo "     $ /mobile-e2e android"
echo ""
echo "  2. Run command:"
if [ -n "$TEST_NAME" ]; then
    echo "     $ agent-mobile $TEST_NAME [args]"
else
    echo "     $ agent-mobile <command> [args]"
fi
echo ""
echo "  3. Visually verify result in emulator"
echo ""
echo "  4. Save evidence:"
echo "     $ agent-mobile screenshot /tmp/evidence.png"
echo ""

echo "Checklist:"
echo "  [ ] Command executes without errors (normal case)"
echo "  [ ] Error messages are clear (error case)"
echo "  [ ] UI interaction results are visually correct"
echo "  [ ] Screenshot evidence saved to /tmp/"
echo "  [ ] XCUITest Runner / ADB operations complete without timeout"
echo ""

echo "For detailed testing strategies, see: references/testing-guide.md"
echo ""

echo "=========================================="
echo " Summary"
echo "=========================================="
echo ""
echo "1. Unit tests      → cargo test --verbose --bins"
echo "2. Integration     → cargo test --test cli -- --test-threads=1"
echo "3. Real device ✓   → /mobile-e2e ios|android (REQUIRED!)"
echo ""
echo "After all tests pass → Ready to commit!"
