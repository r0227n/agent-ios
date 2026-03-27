#!/bin/bash
#
# Multi-Device Test Template
#
# This script runs the same test across multiple devices (iOS and Android)
# in parallel, demonstrating cross-platform testing capabilities.
#
# It showcases:
# - Session-based device management
# - Parallel test execution
# - Cross-platform compatibility
# - Result aggregation and comparison
#
# Usage:
#   ./multi-device-test.sh <bundle-id>
#
# Example:
#   ./multi-device-test.sh com.example.app

set -e  # Exit on error

# Configuration
BUNDLE_ID="${1:-com.example.app}"
OUTPUT_DIR="/tmp/agent-mobile-multi-device"
RESULTS_DIR="$OUTPUT_DIR/results"
REPORT_FILE="$OUTPUT_DIR/test_report.txt"

# Test configuration
TEST_EMAIL="test@example.com"
TEST_PASSWORD="password123"

echo "=== Multi-Device Test Workflow ==="
echo "App: $BUNDLE_ID"
echo "Output: $OUTPUT_DIR"
echo ""

# Setup
mkdir -p "$OUTPUT_DIR" "$RESULTS_DIR"
: > "$REPORT_FILE"

# Discover available devices
echo "[1/7] Discovering available devices..."
IOS_DEVICES=$(agent-mobile device list -p ios -f json 2>/dev/null || echo "[]")
ANDROID_DEVICES=$(agent-mobile device list -p android -f json 2>/dev/null || echo "[]")

# Extract booted iOS devices
IOS_COUNT=0
if command -v jq &> /dev/null; then
  IOS_COUNT=$(echo "$IOS_DEVICES" | jq '[.[] | select(.state == "Booted")] | length')
  ANDROID_COUNT=$(echo "$ANDROID_DEVICES" | jq '[.[] | select(.state == "device")] | length')
else
  echo "Warning: jq not installed - device detection may be limited"
  IOS_COUNT=$(echo "$IOS_DEVICES" | grep -c "Booted" || echo "0")
  ANDROID_COUNT=$(echo "$ANDROID_DEVICES" | grep -c "device" || echo "0")
fi

echo "  iOS devices (booted): $IOS_COUNT"
echo "  Android devices (connected): $ANDROID_COUNT"

if [ $IOS_COUNT -eq 0 ] && [ $ANDROID_COUNT -eq 0 ]; then
  echo "Error: No devices available"
  echo "Please boot at least one iOS simulator or Android emulator"
  exit 1
fi

# Step 2: Create sessions
echo "[2/7] Creating sessions..."
SESSIONS=()

# Create iOS sessions
if [ $IOS_COUNT -gt 0 ] && command -v jq &> /dev/null; then
  echo "$IOS_DEVICES" | jq -r '.[] | select(.state == "Booted") | "\(.name)|\(.udid)"' | \
  while IFS='|' read -r name udid; do
    session_name=$(echo "ios-${name}" | tr ' ' '-' | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]//g')
    agent-mobile session create "$session_name" --udid "$udid" -p ios 2>/dev/null || true
    echo "  Created session: $session_name (iOS)"
    SESSIONS+=("$session_name")
  done
fi

# Create Android sessions
if [ $ANDROID_COUNT -gt 0 ] && command -v jq &> /dev/null; then
  echo "$ANDROID_DEVICES" | jq -r '.[] | select(.state == "device") | "\(.name)|\(.udid)"' | \
  while IFS='|' read -r name udid; do
    session_name=$(echo "android-${name}" | tr ' ' '-' | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]//g')
    agent-mobile session create "$session_name" --udid "$udid" -p android 2>/dev/null || true
    echo "  Created session: $session_name (Android)"
    SESSIONS+=("$session_name")
  done
fi

# Get session list for execution
SESSIONS=($(agent-mobile session list -f json 2>/dev/null | \
           jq -r '.[].name' 2>/dev/null || \
           agent-mobile session list 2>/dev/null | awk '{print $1}' | tail -n +3))

if [ ${#SESSIONS[@]} -eq 0 ]; then
  echo "Error: No sessions created"
  exit 1
fi

echo "  Total sessions: ${#SESSIONS[@]}"

# Step 3: Define test function
run_test() {
  local session=$1
  local result_file="$RESULTS_DIR/${session}_result.txt"
  local screenshot_dir="$RESULTS_DIR/${session}_screenshots"

  mkdir -p "$screenshot_dir"

  echo "=== Testing on $session ===" > "$result_file"
  echo "Start time: $(date)" >> "$result_file"
  echo "" >> "$result_file"

  # Test execution
  {
    # 1. Launch app
    echo "[1/6] Launching app..." | tee -a "$result_file"
    agent-mobile --session "$session" app launch "$BUNDLE_ID"
    sleep 2

    # 2. Take initial screenshot
    echo "[2/6] Taking initial screenshot..." | tee -a "$result_file"
    agent-mobile --session "$session" screenshot -o "$screenshot_dir/01_initial.png"

    # 3. Capture UI snapshot
    echo "[3/6] Capturing UI snapshot..." | tee -a "$result_file"
    agent-mobile --session "$session" snapshot -i > "$screenshot_dir/snapshot.txt"

    # 4. Login flow
    echo "[4/6] Performing login..." | tee -a "$result_file"

    # Try semantic locators (cross-platform)
    if agent-mobile --session "$session" find label "Email" fill "$TEST_EMAIL" 2>/dev/null; then
      echo "  ✓ Filled email field" | tee -a "$result_file"
    else
      echo "  ✗ Failed to find email field" | tee -a "$result_file"
    fi

    if agent-mobile --session "$session" find label "Password" fill "$TEST_PASSWORD" 2>/dev/null; then
      echo "  ✓ Filled password field" | tee -a "$result_file"
    else
      echo "  ✗ Failed to find password field" | tee -a "$result_file"
    fi

    agent-mobile --session "$session" screenshot -o "$screenshot_dir/02_filled.png"

    # 5. Submit
    echo "[5/6] Submitting form..." | tee -a "$result_file"
    if agent-mobile --session "$session" find text "Login" tap 2>/dev/null || \
       agent-mobile --session "$session" find text "Sign in" tap 2>/dev/null; then
      echo "  ✓ Tapped login button" | tee -a "$result_file"
    else
      echo "  ✗ Failed to find login button" | tee -a "$result_file"
    fi

    sleep 3

    # 6. Verify
    echo "[6/6] Verifying result..." | tee -a "$result_file"
    agent-mobile --session "$session" screenshot -o "$screenshot_dir/03_result.png"
    agent-mobile --session "$session" snapshot -i > "$screenshot_dir/final_snapshot.txt"

    # Check for success/failure
    if grep -qiE "logout|home|dashboard|welcome" "$screenshot_dir/final_snapshot.txt" 2>/dev/null; then
      echo "  ✓ Login successful" | tee -a "$result_file"
      echo "RESULT: SUCCESS" >> "$result_file"
    else
      echo "  ✗ Login failed" | tee -a "$result_file"
      echo "RESULT: FAILURE" >> "$result_file"
    fi

  } 2>&1 | tee -a "$result_file"

  echo "" >> "$result_file"
  echo "End time: $(date)" >> "$result_file"
}

# Step 4: Run tests in parallel
echo "[3/7] Running tests in parallel..."
echo ""

# Launch tests in background
for session in "${SESSIONS[@]}"; do
  echo "  Starting test on $session..."
  run_test "$session" &
done

# Wait for all tests to complete
echo ""
echo "Waiting for all tests to complete..."
wait

# Step 5: Collect results
echo ""
echo "[4/7] Collecting results..."

SUCCESS_COUNT=0
FAILURE_COUNT=0
TOTAL_COUNT=${#SESSIONS[@]}

echo "=== Test Results ===" > "$REPORT_FILE"
echo "App: $BUNDLE_ID" >> "$REPORT_FILE"
echo "Date: $(date)" >> "$REPORT_FILE"
echo "Total devices: $TOTAL_COUNT" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

for session in "${SESSIONS[@]}"; do
  result_file="$RESULTS_DIR/${session}_result.txt"

  if [ -f "$result_file" ]; then
    if grep -q "RESULT: SUCCESS" "$result_file"; then
      echo "✓ $session: SUCCESS" | tee -a "$REPORT_FILE"
      SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
      echo "✗ $session: FAILURE" | tee -a "$REPORT_FILE"
      FAILURE_COUNT=$((FAILURE_COUNT + 1))
    fi
  else
    echo "⚠ $session: NO RESULT" | tee -a "$REPORT_FILE"
    FAILURE_COUNT=$((FAILURE_COUNT + 1))
  fi
done

echo "" >> "$REPORT_FILE"
echo "Summary:" >> "$REPORT_FILE"
echo "  Success: $SUCCESS_COUNT / $TOTAL_COUNT" >> "$REPORT_FILE"
echo "  Failure: $FAILURE_COUNT / $TOTAL_COUNT" >> "$REPORT_FILE"
echo "  Pass Rate: $(( SUCCESS_COUNT * 100 / TOTAL_COUNT ))%" >> "$REPORT_FILE"

# Step 6: Generate comparison
echo ""
echo "[5/7] Generating comparison..."

if [ $SUCCESS_COUNT -eq $TOTAL_COUNT ]; then
  echo "" >> "$REPORT_FILE"
  echo "🎉 All tests passed!" >> "$REPORT_FILE"
  echo "" >> "$REPORT_FILE"
  echo "Cross-platform compatibility: ✓" >> "$REPORT_FILE"
elif [ $SUCCESS_COUNT -gt 0 ]; then
  echo "" >> "$REPORT_FILE"
  echo "⚠ Partial success - investigate failures" >> "$REPORT_FILE"
  echo "" >> "$REPORT_FILE"
  echo "Failing devices:" >> "$REPORT_FILE"

  for session in "${SESSIONS[@]}"; do
    result_file="$RESULTS_DIR/${session}_result.txt"
    if [ -f "$result_file" ] && ! grep -q "RESULT: SUCCESS" "$result_file"; then
      echo "  - $session" >> "$REPORT_FILE"
      echo "    See: $result_file" >> "$REPORT_FILE"
    fi
  done
else
  echo "" >> "$REPORT_FILE"
  echo "❌ All tests failed" >> "$REPORT_FILE"
fi

# Step 7: Cleanup (optional)
echo ""
echo "[6/7] Cleanup..."
read -p "Destroy test sessions? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  for session in "${SESSIONS[@]}"; do
    agent-mobile session rm "$session" 2>/dev/null || true
  done
  echo "  Sessions removed"
else
  echo "  Sessions preserved for further investigation"
fi

# Final report
echo ""
echo "[7/7] Test complete!"
echo ""
echo "=== Summary ==="
echo "  Total devices tested: $TOTAL_COUNT"
echo "  Successful: $SUCCESS_COUNT"
echo "  Failed: $FAILURE_COUNT"
echo "  Pass rate: $(( SUCCESS_COUNT * 100 / TOTAL_COUNT ))%"
echo ""
echo "Results:"
echo "  - Report: $REPORT_FILE"
echo "  - Screenshots: $RESULTS_DIR/*/screenshots/"
echo "  - Logs: $RESULTS_DIR/*_result.txt"
echo ""
echo "View report:"
echo "  cat $REPORT_FILE"

# Exit with appropriate code
if [ $SUCCESS_COUNT -eq $TOTAL_COUNT ]; then
  exit 0
else
  exit 1
fi
