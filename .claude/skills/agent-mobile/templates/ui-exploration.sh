#!/bin/bash
#
# UI Exploration Template
#
# This script explores the UI structure of an app using different
# snapshot configurations to find the optimal balance between
# completeness and token efficiency.
#
# It demonstrates:
# - Full snapshot vs interactive-only
# - Compact mode and depth limiting
# - JSON output for programmatic analysis
# - Element counting and categorization
#
# Usage:
#   ./ui-exploration.sh <bundle-id>
#
# Example:
#   ./ui-exploration.sh com.example.app

set -e  # Exit on error

# Configuration
BUNDLE_ID="${1:-com.example.app}"
OUTPUT_DIR="/tmp/agent-mobile-exploration"
REPORT_FILE="$OUTPUT_DIR/exploration_report.txt"

echo "=== UI Exploration Workflow ==="
echo "App: $BUNDLE_ID"
echo "Output: $OUTPUT_DIR"
echo ""

# Setup
mkdir -p "$OUTPUT_DIR"
: > "$REPORT_FILE"  # Clear report file

# Helper function to count elements
count_elements() {
  local file=$1
  grep -c "^@e[0-9]*" "$file" || echo "0"
}

# Helper function to categorize elements
categorize_elements() {
  local file=$1
  echo "=== Element Types ===" >> "$REPORT_FILE"
  grep "^@e[0-9]*" "$file" | \
    sed 's/.*@e[0-9]* \([^ ]*\).*/\1/' | \
    sort | uniq -c | sort -rn >> "$REPORT_FILE"
  echo "" >> "$REPORT_FILE"
}

# Step 1: Launch app
echo "[1/8] Launching app..."
agent-mobile app launch "$BUNDLE_ID"
sleep 2

# Step 2: Initial screenshot
echo "[2/8] Taking initial screenshot..."
agent-mobile screenshot -o "$OUTPUT_DIR/initial.png"
echo "  Screenshot: $OUTPUT_DIR/initial.png"

# Step 3: Full snapshot (default)
echo "[3/8] Taking full snapshot (all elements)..."
agent-mobile snapshot > "$OUTPUT_DIR/full_snapshot.txt"
FULL_COUNT=$(count_elements "$OUTPUT_DIR/full_snapshot.txt")
echo "  Elements: $FULL_COUNT"
echo "Full Snapshot: $FULL_COUNT elements" >> "$REPORT_FILE"
categorize_elements "$OUTPUT_DIR/full_snapshot.txt"

# Step 4: Interactive-only snapshot
echo "[4/8] Taking interactive-only snapshot..."
agent-mobile snapshot -i > "$OUTPUT_DIR/interactive_snapshot.txt"
INTERACTIVE_COUNT=$(count_elements "$OUTPUT_DIR/interactive_snapshot.txt")
if [ "$FULL_COUNT" -eq 0 ]; then
  REDUCTION="n/a"
  echo "  Elements: $INTERACTIVE_COUNT (reduction unavailable; full snapshot had 0 elements)"
  echo "Interactive Snapshot (-i): $INTERACTIVE_COUNT elements (reduction unavailable; full snapshot had 0 elements)" >> "$REPORT_FILE"
else
  REDUCTION=$((100 - (INTERACTIVE_COUNT * 100 / FULL_COUNT)))
  echo "  Elements: $INTERACTIVE_COUNT ($REDUCTION% reduction)"
  echo "Interactive Snapshot (-i): $INTERACTIVE_COUNT elements ($REDUCTION% reduction)" >> "$REPORT_FILE"
fi
categorize_elements "$OUTPUT_DIR/interactive_snapshot.txt"

# Step 5: Compact snapshot
echo "[5/8] Taking compact snapshot..."
agent-mobile snapshot -i -c > "$OUTPUT_DIR/compact_snapshot.txt"
COMPACT_COUNT=$(count_elements "$OUTPUT_DIR/compact_snapshot.txt")
if [ "$FULL_COUNT" -eq 0 ]; then
  COMPACT_REDUCTION="n/a"
  echo "  Elements: $COMPACT_COUNT (reduction unavailable; full snapshot had 0 elements)"
  echo "Compact Snapshot (-i -c): $COMPACT_COUNT elements (reduction unavailable; full snapshot had 0 elements)" >> "$REPORT_FILE"
else
  COMPACT_REDUCTION=$((100 - (COMPACT_COUNT * 100 / FULL_COUNT)))
  echo "  Elements: $COMPACT_COUNT ($COMPACT_REDUCTION% reduction)"
  echo "Compact Snapshot (-i -c): $COMPACT_COUNT elements ($COMPACT_REDUCTION% reduction)" >> "$REPORT_FILE"
fi
categorize_elements "$OUTPUT_DIR/compact_snapshot.txt"

# Step 6: Depth-limited snapshot
echo "[6/8] Taking depth-limited snapshot (depth=3)..."
agent-mobile snapshot -i -c -d 3 > "$OUTPUT_DIR/depth_limited_snapshot.txt"
DEPTH_COUNT=$(count_elements "$OUTPUT_DIR/depth_limited_snapshot.txt")
if [ "$FULL_COUNT" -eq 0 ]; then
  DEPTH_REDUCTION="n/a"
  echo "  Elements: $DEPTH_COUNT (reduction unavailable; full snapshot had 0 elements)"
  echo "Depth-Limited Snapshot (-i -c -d 3): $DEPTH_COUNT elements (reduction unavailable; full snapshot had 0 elements)" >> "$REPORT_FILE"
else
  DEPTH_REDUCTION=$((100 - (DEPTH_COUNT * 100 / FULL_COUNT)))
  echo "  Elements: $DEPTH_COUNT ($DEPTH_REDUCTION% reduction)"
  echo "Depth-Limited Snapshot (-i -c -d 3): $DEPTH_COUNT elements ($DEPTH_REDUCTION% reduction)" >> "$REPORT_FILE"
fi
categorize_elements "$OUTPUT_DIR/depth_limited_snapshot.txt"

# Step 7: JSON snapshot for analysis
echo "[7/8] Taking JSON snapshot for programmatic analysis..."
agent-mobile snapshot -i -f json > "$OUTPUT_DIR/snapshot.json"

# Analyze JSON structure
echo "" >> "$REPORT_FILE"
echo "=== JSON Analysis ===" >> "$REPORT_FILE"

# Count by element type using jq
if command -v jq &> /dev/null; then
  echo "Element counts by type:" >> "$REPORT_FILE"
  jq -r '.elements[] | .element_type' "$OUTPUT_DIR/snapshot.json" | \
    sort | uniq -c | sort -rn >> "$REPORT_FILE"

  echo "" >> "$REPORT_FILE"
  echo "Enabled vs Disabled:" >> "$REPORT_FILE"
  ENABLED=$(jq '[.elements[] | select(.enabled == true)] | length' "$OUTPUT_DIR/snapshot.json")
  DISABLED=$(jq '[.elements[] | select(.enabled == false)] | length' "$OUTPUT_DIR/snapshot.json")
  echo "  Enabled: $ENABLED" >> "$REPORT_FILE"
  echo "  Disabled: $DISABLED" >> "$REPORT_FILE"

  echo "" >> "$REPORT_FILE"
  echo "Elements with text content:" >> "$REPORT_FILE"
  WITH_LABEL=$(jq '[.elements[] | select(.label != null and .label != "")] | length' "$OUTPUT_DIR/snapshot.json")
  WITH_VALUE=$(jq '[.elements[] | select(.value != null and .value != "")] | length' "$OUTPUT_DIR/snapshot.json")
  WITH_PLACEHOLDER=$(jq '[.elements[] | select(.placeholder != null and .placeholder != "")] | length' "$OUTPUT_DIR/snapshot.json")
  echo "  With label: $WITH_LABEL" >> "$REPORT_FILE"
  echo "  With value: $WITH_VALUE" >> "$REPORT_FILE"
  echo "  With placeholder: $WITH_PLACEHOLDER" >> "$REPORT_FILE"

  echo "" >> "$REPORT_FILE"
  echo "Top 10 elements by frame size:" >> "$REPORT_FILE"
  jq -r '.elements[] | "\(.frame.width * .frame.height) \(.ref) \(.element_type) \(.label // "")"' \
    "$OUTPUT_DIR/snapshot.json" | \
    sort -rn | head -10 >> "$REPORT_FILE"
else
  echo "jq not installed - skipping JSON analysis" >> "$REPORT_FILE"
fi

# Step 8: Element reference mapping
echo "[8/8] Creating element reference mapping..."
echo "" >> "$REPORT_FILE"
echo "=== Element Reference Mapping ===" >> "$REPORT_FILE"
echo "Top 20 interactive elements:" >> "$REPORT_FILE"
head -20 "$OUTPUT_DIR/interactive_snapshot.txt" >> "$REPORT_FILE"

# Generate summary
echo ""
echo "=== Exploration Complete ==="
echo ""
echo "Summary:"
echo "  Full snapshot:        $FULL_COUNT elements"
echo "  Interactive only:     $INTERACTIVE_COUNT elements ($REDUCTION% reduction)"
echo "  Compact:              $COMPACT_COUNT elements ($COMPACT_REDUCTION% reduction)"
echo "  Depth-limited (3):    $DEPTH_COUNT elements ($DEPTH_REDUCTION% reduction)"
echo ""
echo "Recommendation:"
if [ $DEPTH_COUNT -lt 20 ]; then
  echo "  ✓ Use: agent-mobile snapshot -i -c -d 3"
  echo "    Optimal balance: only $DEPTH_COUNT elements"
elif [ $COMPACT_COUNT -lt 50 ]; then
  echo "  ✓ Use: agent-mobile snapshot -i -c"
  echo "    Good balance: $COMPACT_COUNT elements"
elif [ $INTERACTIVE_COUNT -lt 100 ]; then
  echo "  ✓ Use: agent-mobile snapshot -i"
  echo "    Reasonable: $INTERACTIVE_COUNT elements"
else
  echo "  ⚠ Complex UI: Consider using specific element queries"
  echo "    Full interactive snapshot has $INTERACTIVE_COUNT elements"
fi
echo ""
echo "Files generated:"
echo "  - $OUTPUT_DIR/full_snapshot.txt"
echo "  - $OUTPUT_DIR/interactive_snapshot.txt"
echo "  - $OUTPUT_DIR/compact_snapshot.txt"
echo "  - $OUTPUT_DIR/depth_limited_snapshot.txt"
echo "  - $OUTPUT_DIR/snapshot.json"
echo "  - $OUTPUT_DIR/initial.png"
echo "  - $REPORT_FILE"
echo ""
echo "View report:"
echo "  cat $REPORT_FILE"
