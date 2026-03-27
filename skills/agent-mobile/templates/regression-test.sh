#!/bin/bash
#
# Regression Test Template
#
# This script performs regression testing by comparing UI snapshots
# and screenshots before and after code changes.
#
# It demonstrates:
# - Baseline capture and storage
# - Current state capture
# - Structural comparison (snapshot diff)
# - Visual comparison (screenshot diff)
# - Change reporting
#
# Usage:
#   # 1. Capture baseline (before changes)
#   ./regression-test.sh capture <bundle-id> <baseline-name>
#
#   # 2. Make code changes and rebuild app
#
#   # 3. Compare against baseline (after changes)
#   ./regression-test.sh compare <bundle-id> <baseline-name>
#
# Example:
#   ./regression-test.sh capture com.example.app v1.0.0
#   # ... make changes ...
#   ./regression-test.sh compare com.example.app v1.0.0

set -e  # Exit on error

# Configuration
COMMAND="${1:-capture}"
BUNDLE_ID="${2:-com.example.app}"
BASELINE_NAME="${3:-baseline}"
BASELINE_DIR="/tmp/agent-mobile-baselines/$BASELINE_NAME"
CURRENT_DIR="/tmp/agent-mobile-current"
REPORT_FILE="/tmp/agent-mobile-regression-report.txt"

# Colors for output (if terminal supports it)
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  RED='\033[0;31m'
  YELLOW='\033[1;33m'
  NC='\033[0m' # No Color
else
  GREEN=''
  RED=''
  YELLOW=''
  NC=''
fi

# Function: Capture UI state
capture_state() {
  local output_dir=$1
  local label=$2

  echo "Capturing state: $label"
  mkdir -p "$output_dir/screenshots"

  # Launch app
  echo "  [1/5] Launching app..."
  agent-mobile app launch "$BUNDLE_ID"
  sleep 2

  # Main screen
  echo "  [2/5] Capturing main screen..."
  agent-mobile screenshot -o "$output_dir/screenshots/main.png"
  agent-mobile snapshot -i -f json > "$output_dir/main_snapshot.json"
  agent-mobile snapshot -i > "$output_dir/main_snapshot.txt"

  # Navigate to different screens (customize based on your app)
  echo "  [3/5] Exploring UI..."

  # Try to find and tap common navigation elements
  if agent-mobile find text "Settings" tap 2>/dev/null || \
     agent-mobile find text "Menu" tap 2>/dev/null; then
    sleep 1
    agent-mobile screenshot -o "$output_dir/screenshots/screen2.png"
    agent-mobile snapshot -i -f json > "$output_dir/screen2_snapshot.json"
    agent-mobile tap back 2>/dev/null || agent-mobile swipe right 2>/dev/null || true
    sleep 1
  fi

  # Capture element statistics
  echo "  [4/5] Analyzing UI structure..."
  if command -v jq &> /dev/null; then
    # Count elements by type
    jq -r '.elements[] | .element_type' "$output_dir/main_snapshot.json" | \
      sort | uniq -c | sort -rn > "$output_dir/element_counts.txt"

    # Extract all interactive elements
    jq -r '.elements[] | "\(.ref) \(.element_type) \(.label // "") \(.value // "")"' \
      "$output_dir/main_snapshot.json" > "$output_dir/elements_list.txt"

    # Count total elements
    jq '.elements | length' "$output_dir/main_snapshot.json" > "$output_dir/total_count.txt"
  fi

  # Create metadata
  echo "  [5/5] Creating metadata..."
  cat > "$output_dir/metadata.txt" <<EOF
Capture Date: $(date)
Bundle ID: $BUNDLE_ID
Label: $label
Device: $(agent-mobile device list | head -3 | tail -1 || echo "Unknown")
EOF

  echo "  ✓ State captured: $output_dir"
}

# Function: Compare states
compare_states() {
  echo ""
  echo "=== Regression Test Report ===" > "$REPORT_FILE"
  echo "Baseline: $BASELINE_NAME" >> "$REPORT_FILE"
  echo "Date: $(date)" >> "$REPORT_FILE"
  echo "Bundle ID: $BUNDLE_ID" >> "$REPORT_FILE"
  echo "" >> "$REPORT_FILE"

  # Compare element counts
  echo "=== Element Count Comparison ===" >> "$REPORT_FILE"

  if [ -f "$BASELINE_DIR/total_count.txt" ] && [ -f "$CURRENT_DIR/total_count.txt" ]; then
    BASELINE_COUNT=$(cat "$BASELINE_DIR/total_count.txt")
    CURRENT_COUNT=$(cat "$CURRENT_DIR/total_count.txt")
    DIFF=$((CURRENT_COUNT - BASELINE_COUNT))

    echo "Baseline elements: $BASELINE_COUNT" >> "$REPORT_FILE"
    echo "Current elements:  $CURRENT_COUNT" >> "$REPORT_FILE"

    if [ $DIFF -eq 0 ]; then
      echo "Difference: 0 (no change)" >> "$REPORT_FILE"
      echo -e "${GREEN}✓ Element count unchanged${NC}"
    elif [ $DIFF -gt 0 ]; then
      echo "Difference: +$DIFF (increase)" >> "$REPORT_FILE"
      echo -e "${YELLOW}⚠ Element count increased by $DIFF${NC}"
    else
      echo "Difference: $DIFF (decrease)" >> "$REPORT_FILE"
      echo -e "${YELLOW}⚠ Element count decreased by ${DIFF#-}${NC}"
    fi
  else
    echo "Unable to compare element counts" >> "$REPORT_FILE"
  fi

  echo "" >> "$REPORT_FILE"

  # Compare element types
  echo "=== Element Type Changes ===" >> "$REPORT_FILE"

  if [ -f "$BASELINE_DIR/element_counts.txt" ] && [ -f "$CURRENT_DIR/element_counts.txt" ]; then
    # Show differences in element types
    diff "$BASELINE_DIR/element_counts.txt" "$CURRENT_DIR/element_counts.txt" >> "$REPORT_FILE" 2>&1 || true

    if diff -q "$BASELINE_DIR/element_counts.txt" "$CURRENT_DIR/element_counts.txt" > /dev/null 2>&1; then
      echo "No changes in element type distribution" >> "$REPORT_FILE"
      echo -e "${GREEN}✓ Element types unchanged${NC}"
    else
      echo -e "${YELLOW}⚠ Element type distribution changed${NC}"
    fi
  else
    echo "Unable to compare element types" >> "$REPORT_FILE"
  fi

  echo "" >> "$REPORT_FILE"

  # Compare specific elements
  echo "=== Element Additions/Removals ===" >> "$REPORT_FILE"

  if [ -f "$BASELINE_DIR/elements_list.txt" ] && [ -f "$CURRENT_DIR/elements_list.txt" ]; then
    # Find added elements
    ADDED=$(comm -13 \
      <(sort "$BASELINE_DIR/elements_list.txt") \
      <(sort "$CURRENT_DIR/elements_list.txt") | wc -l)

    # Find removed elements
    REMOVED=$(comm -23 \
      <(sort "$BASELINE_DIR/elements_list.txt") \
      <(sort "$CURRENT_DIR/elements_list.txt") | wc -l)

    echo "Added elements: $ADDED" >> "$REPORT_FILE"
    echo "Removed elements: $REMOVED" >> "$REPORT_FILE"

    if [ $ADDED -gt 0 ]; then
      echo "" >> "$REPORT_FILE"
      echo "New elements:" >> "$REPORT_FILE"
      comm -13 \
        <(sort "$BASELINE_DIR/elements_list.txt") \
        <(sort "$CURRENT_DIR/elements_list.txt") | head -10 >> "$REPORT_FILE"
      if [ $ADDED -gt 10 ]; then
        echo "... and $((ADDED - 10)) more" >> "$REPORT_FILE"
      fi
    fi

    if [ $REMOVED -gt 0 ]; then
      echo "" >> "$REPORT_FILE"
      echo "Removed elements:" >> "$REPORT_FILE"
      comm -23 \
        <(sort "$BASELINE_DIR/elements_list.txt") \
        <(sort "$CURRENT_DIR/elements_list.txt") | head -10 >> "$REPORT_FILE"
      if [ $REMOVED -gt 10 ]; then
        echo "... and $((REMOVED - 10)) more" >> "$REPORT_FILE"
      fi
    fi

    if [ $ADDED -eq 0 ] && [ $REMOVED -eq 0 ]; then
      echo -e "${GREEN}✓ No element additions or removals${NC}"
    else
      echo -e "${YELLOW}⚠ $ADDED elements added, $REMOVED elements removed${NC}"
    fi
  else
    echo "Unable to compare specific elements" >> "$REPORT_FILE"
  fi

  echo "" >> "$REPORT_FILE"

  # Visual comparison (if ImageMagick is available)
  echo "=== Visual Comparison ===" >> "$REPORT_FILE"

  if command -v compare &> /dev/null; then
    if [ -f "$BASELINE_DIR/screenshots/main.png" ] && [ -f "$CURRENT_DIR/screenshots/main.png" ]; then
      echo "Generating visual diff..." | tee -a "$REPORT_FILE"

      DIFF_DIR="/tmp/agent-mobile-visual-diff"
      mkdir -p "$DIFF_DIR"

      compare "$BASELINE_DIR/screenshots/main.png" \
              "$CURRENT_DIR/screenshots/main.png" \
              "$DIFF_DIR/main_diff.png" 2>/dev/null || true

      if [ -f "$DIFF_DIR/main_diff.png" ]; then
        echo "Visual diff saved: $DIFF_DIR/main_diff.png" >> "$REPORT_FILE"
        echo -e "${GREEN}✓ Visual diff generated${NC}"
      else
        echo "Identical screenshots" >> "$REPORT_FILE"
        echo -e "${GREEN}✓ Screenshots identical${NC}"
      fi
    else
      echo "Missing screenshots for comparison" >> "$REPORT_FILE"
    fi
  else
    echo "ImageMagick 'compare' not available - skipping visual diff" >> "$REPORT_FILE"
    echo "Install: brew install imagemagick (macOS) or apt install imagemagick (Linux)" >> "$REPORT_FILE"
  fi

  echo "" >> "$REPORT_FILE"

  # JSON structure comparison
  echo "=== JSON Structure Comparison ===" >> "$REPORT_FILE"

  if command -v jq &> /dev/null; then
    if [ -f "$BASELINE_DIR/main_snapshot.json" ] && [ -f "$CURRENT_DIR/main_snapshot.json" ]; then
      # Compare using jq
      BASELINE_KEYS=$(jq -r '.elements[0] | keys[]' "$BASELINE_DIR/main_snapshot.json" 2>/dev/null | sort)
      CURRENT_KEYS=$(jq -r '.elements[0] | keys[]' "$CURRENT_DIR/main_snapshot.json" 2>/dev/null | sort)

      if [ "$BASELINE_KEYS" = "$CURRENT_KEYS" ]; then
        echo "JSON structure unchanged" >> "$REPORT_FILE"
        echo -e "${GREEN}✓ JSON structure unchanged${NC}"
      else
        echo "JSON structure changed" >> "$REPORT_FILE"
        echo -e "${YELLOW}⚠ JSON structure changed${NC}"
      fi
    fi
  fi

  echo "" >> "$REPORT_FILE"

  # Overall verdict
  echo "=== Verdict ===" >> "$REPORT_FILE"

  CHANGES=0
  if [ $DIFF -ne 0 ] 2>/dev/null; then CHANGES=$((CHANGES + 1)); fi
  if [ $ADDED -gt 0 ] 2>/dev/null; then CHANGES=$((CHANGES + 1)); fi
  if [ $REMOVED -gt 0 ] 2>/dev/null; then CHANGES=$((CHANGES + 1)); fi

  if [ $CHANGES -eq 0 ]; then
    echo "No significant changes detected" >> "$REPORT_FILE"
    echo -e "${GREEN}✓ PASS: No regressions detected${NC}" >> "$REPORT_FILE"
  elif [ $CHANGES -le 2 ]; then
    echo "Minor changes detected - review recommended" >> "$REPORT_FILE"
    echo -e "${YELLOW}⚠ WARN: Minor changes detected${NC}" >> "$REPORT_FILE"
  else
    echo "Significant changes detected - thorough review required" >> "$REPORT_FILE"
    echo -e "${RED}✗ FAIL: Significant changes detected${NC}" >> "$REPORT_FILE"
  fi

  echo "" >> "$REPORT_FILE"
  echo "Baseline: $BASELINE_DIR" >> "$REPORT_FILE"
  echo "Current:  $CURRENT_DIR" >> "$REPORT_FILE"
  echo "Report:   $REPORT_FILE" >> "$REPORT_FILE"
}

# Main execution
case "$COMMAND" in
  capture)
    echo "=== Capturing Baseline ==="
    echo "Bundle ID: $BUNDLE_ID"
    echo "Baseline Name: $BASELINE_NAME"
    echo ""

    if [ -d "$BASELINE_DIR" ]; then
      echo "Warning: Baseline '$BASELINE_NAME' already exists"
      read -p "Overwrite? (y/n) " -n 1 -r
      echo
      if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted"
        exit 1
      fi
      rm -rf "$BASELINE_DIR"
    fi

    capture_state "$BASELINE_DIR" "$BASELINE_NAME"

    echo ""
    echo "✓ Baseline captured successfully"
    echo "Location: $BASELINE_DIR"
    echo ""
    echo "Next steps:"
    echo "  1. Make your code changes"
    echo "  2. Rebuild the app"
    echo "  3. Run: $0 compare $BUNDLE_ID $BASELINE_NAME"
    ;;

  compare)
    echo "=== Running Regression Test ==="
    echo "Bundle ID: $BUNDLE_ID"
    echo "Baseline: $BASELINE_NAME"
    echo ""

    if [ ! -d "$BASELINE_DIR" ]; then
      echo "Error: Baseline '$BASELINE_NAME' not found"
      echo "Run: $0 capture $BUNDLE_ID $BASELINE_NAME"
      exit 1
    fi

    # Capture current state
    rm -rf "$CURRENT_DIR"
    capture_state "$CURRENT_DIR" "current"

    echo ""
    echo "Comparing states..."
    compare_states

    # Display report
    echo ""
    cat "$REPORT_FILE"

    echo ""
    echo "Full report: $REPORT_FILE"

    # Exit with appropriate code
    if grep -q "PASS" "$REPORT_FILE"; then
      exit 0
    elif grep -q "WARN" "$REPORT_FILE"; then
      exit 1
    else
      exit 2
    fi
    ;;

  list)
    echo "=== Available Baselines ==="
    if [ -d "/tmp/agent-mobile-baselines" ]; then
      ls -1 /tmp/agent-mobile-baselines/ | while read -r baseline; do
        metadata="/tmp/agent-mobile-baselines/$baseline/metadata.txt"
        if [ -f "$metadata" ]; then
          echo ""
          echo "Baseline: $baseline"
          cat "$metadata" | sed 's/^/  /'
        fi
      done
    else
      echo "No baselines found"
    fi
    ;;

  *)
    echo "Usage: $0 <command> <bundle-id> <baseline-name>"
    echo ""
    echo "Commands:"
    echo "  capture <bundle-id> <baseline-name>  - Capture baseline state"
    echo "  compare <bundle-id> <baseline-name>  - Compare against baseline"
    echo "  list                                 - List available baselines"
    echo ""
    echo "Example:"
    echo "  $0 capture com.example.app v1.0.0"
    echo "  $0 compare com.example.app v1.0.0"
    exit 1
    ;;
esac
