#!/bin/bash
#
# Form Automation Template
#
# This script demonstrates advanced form automation using agent-mobile.
# It showcases:
# - Dynamic form field detection
# - Validation before submission
# - Error handling
# - Screenshot evidence collection
#
# Usage:
#   ./form-automation.sh <bundle-id>
#
# Example:
#   ./form-automation.sh com.example.app

set -e  # Exit on error

# Configuration
BUNDLE_ID="${1:-com.example.app}"
OUTPUT_DIR="/tmp/agent-mobile-forms"
SNAPSHOT_FILE="$OUTPUT_DIR/form_snapshot.txt"
SCREENSHOT_DIR="$OUTPUT_DIR/screenshots"

# Form data (customize for your form)
declare -A FORM_DATA=(
  ["First Name"]="John"
  ["Last Name"]="Doe"
  ["Email"]="john.doe@example.com"
  ["Phone"]="555-1234"
  ["Address"]="123 Main St"
  ["City"]="San Francisco"
  ["Zip"]="94102"
)

echo "=== Form Automation Workflow ==="
echo "App: $BUNDLE_ID"
echo "Output: $OUTPUT_DIR"
echo ""

# Setup: Create output directories
mkdir -p "$OUTPUT_DIR" "$SCREENSHOT_DIR"

# Step 1: Launch app
echo "[1/7] Launching app..."
agent-mobile app launch "$BUNDLE_ID"
sleep 2

# Step 2: Take initial screenshot
echo "[2/7] Taking initial screenshot..."
agent-mobile screenshot -o "$SCREENSHOT_DIR/01_initial.png"
echo "  Screenshot: $SCREENSHOT_DIR/01_initial.png"

# Step 3: Capture form snapshot
echo "[3/7] Capturing form snapshot..."
agent-mobile snapshot -i > "$SNAPSHOT_FILE"
echo "  Snapshot: $SNAPSHOT_FILE"

# Step 4: Analyze form fields
echo "[4/7] Analyzing form fields..."

# Extract all TextField/EditText refs and labels
declare -A FIELD_REFS
while IFS= read -r line; do
  # Match lines like: @e1 TextField "Email" (enabled)
  if echo "$line" | grep -qE '@e[0-9]+ (TextField|EditText|SecureTextField)'; then
    ref=$(echo "$line" | grep -o '@e[0-9]*')
    label=$(echo "$line" | sed -n 's/.*"\(.*\)".*/\1/p')

    if [ -n "$label" ]; then
      FIELD_REFS["$label"]="$ref"
      echo "  Found field: $label -> $ref"
    fi
  fi
done < "$SNAPSHOT_FILE"

if [ ${#FIELD_REFS[@]} -eq 0 ]; then
  echo "Error: No form fields found"
  echo "Please check $SNAPSHOT_FILE for available elements"
  exit 1
fi

# Step 5: Fill form fields
echo "[5/7] Filling form fields..."

filled_count=0
for field_name in "${!FORM_DATA[@]}"; do
  value="${FORM_DATA[$field_name]}"

  # Try exact match first
  if [ -n "${FIELD_REFS[$field_name]}" ]; then
    ref="${FIELD_REFS[$field_name]}"
    echo "  Filling '$field_name' with '$value' ($ref)"
    agent-mobile fill "$ref" "$value"
    filled_count=$((filled_count + 1))
  else
    # Try partial match
    matching_ref=""
    for label in "${!FIELD_REFS[@]}"; do
      if echo "$label" | grep -iq "$field_name"; then
        matching_ref="${FIELD_REFS[$label]}"
        break
      fi
    done

    if [ -n "$matching_ref" ]; then
      echo "  Filling '$field_name' (matched: '$label') with '$value' ($matching_ref)"
      agent-mobile fill "$matching_ref" "$value"
      filled_count=$((filled_count + 1))
    else
      echo "  ⚠ Warning: Field '$field_name' not found in form"
    fi
  fi

  # Small delay between fields
  sleep 0.3
done

echo "  Filled $filled_count / ${#FORM_DATA[@]} fields"

# Take screenshot after filling
agent-mobile screenshot -o "$SCREENSHOT_DIR/02_filled.png"
echo "  Screenshot: $SCREENSHOT_DIR/02_filled.png"

# Step 6: Validate and submit
echo "[6/7] Validating and submitting..."

# Re-capture snapshot to get submit button
agent-mobile snapshot -i > "$SNAPSHOT_FILE"

# Find submit button
submit_ref=$(grep -iE "submit|send|save|confirm|next|continue" "$SNAPSHOT_FILE" | \
             grep -o "@e[0-9]*" | head -1)

if [ -z "$submit_ref" ]; then
  echo "Error: Submit button not found"
  echo "Please check $SNAPSHOT_FILE for available buttons"
  exit 1
fi

echo "  Found submit button: $submit_ref"

# Check if submit button is enabled
if agent-mobile is "$submit_ref" enabled; then
  echo "  Submit button is enabled"
  agent-mobile tap "$submit_ref"
  echo "  ✓ Form submitted"
else
  echo "  ⚠ Submit button is disabled"
  echo "  This may indicate validation errors"
  agent-mobile screenshot -o "$SCREENSHOT_DIR/03_validation_error.png"
  exit 1
fi

# Step 7: Wait and verify
echo "[7/7] Waiting for submission to complete..."
sleep 3

# Take final screenshot
agent-mobile screenshot -o "$SCREENSHOT_DIR/04_result.png"
echo "  Screenshot: $SCREENSHOT_DIR/04_result.png"

# Capture final state
agent-mobile snapshot -i > "$OUTPUT_DIR/final_snapshot.txt"

# Check for success indicators
if grep -qiE "success|thank you|confirmed|completed" "$OUTPUT_DIR/final_snapshot.txt"; then
  echo ""
  echo "✓ Form submission successful!"
  echo ""
  echo "Evidence:"
  echo "  - Initial state: $SCREENSHOT_DIR/01_initial.png"
  echo "  - Filled form: $SCREENSHOT_DIR/02_filled.png"
  echo "  - Result: $SCREENSHOT_DIR/04_result.png"
  echo "  - Snapshots: $OUTPUT_DIR/*.txt"
  exit 0
else
  echo ""
  echo "⚠ Form submission may have failed"
  echo "Please review screenshots in: $SCREENSHOT_DIR"

  # Check for error messages
  if grep -qiE "error|invalid|required|missing" "$OUTPUT_DIR/final_snapshot.txt"; then
    echo ""
    echo "Detected error messages:"
    grep -iE "error|invalid|required|missing" "$OUTPUT_DIR/final_snapshot.txt" | head -5
  fi

  exit 1
fi
