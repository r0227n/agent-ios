#!/bin/bash
#
# Login Workflow Template
#
# This script demonstrates a basic login workflow using agent-mobile.
# It showcases the recommended pattern: snapshot → fill → tap → verify.
#
# Usage:
#   ./login-workflow.sh <bundle-id> <email> <password>
#
# Example:
#   ./login-workflow.sh com.example.app test@example.com password123

set -e  # Exit on error

# Check arguments
if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <bundle-id> <email> <password>"
  echo "Example: $0 com.example.app test@example.com password123"
  exit 1
fi

BUNDLE_ID="$1"
EMAIL="$2"
PASSWORD="$3"

echo "=== Login Workflow ==="
echo "App: $BUNDLE_ID"
echo "Email: $EMAIL"
echo ""

# Step 1: Launch the app
echo "[1/6] Launching app..."
agent-mobile app launch "$BUNDLE_ID"

# Step 2: Wait for app to load
echo "[2/6] Waiting for app to load..."
sleep 2

# Step 3: Take UI snapshot
echo "[3/6] Taking UI snapshot..."
agent-mobile snapshot -i > /tmp/login_screen.txt
echo "Snapshot saved to /tmp/login_screen.txt"

# Step 4: Fill login form
echo "[4/6] Filling login form..."

# Method 1: Using semantic locators (readable but token-heavy)
# agent-mobile find label "Email" fill "$EMAIL"
# agent-mobile find label "Password" fill "$PASSWORD"

# Method 2: Using element references (token-efficient)
# Extract refs from snapshot
EMAIL_REF=$(grep -i "email\|username" /tmp/login_screen.txt | grep -o "@e[0-9]*" | head -1)
PASSWORD_REF=$(grep -i "password" /tmp/login_screen.txt | grep -o "@e[0-9]*" | head -1)
LOGIN_BTN_REF=$(grep -i "login\|sign in\|submit" /tmp/login_screen.txt | grep -o "@e[0-9]*" | head -1)

if [ -z "$EMAIL_REF" ] || [ -z "$PASSWORD_REF" ] || [ -z "$LOGIN_BTN_REF" ]; then
  echo "Error: Could not find login form elements"
  echo "Please check /tmp/login_screen.txt for available elements"
  exit 1
fi

echo "  Email field: $EMAIL_REF"
echo "  Password field: $PASSWORD_REF"
echo "  Login button: $LOGIN_BTN_REF"

agent-mobile fill "$EMAIL_REF" "$EMAIL"
agent-mobile fill "$PASSWORD_REF" "$PASSWORD"

# Step 5: Tap login button
echo "[5/6] Tapping login button..."
agent-mobile tap "$LOGIN_BTN_REF"

# Step 6: Wait and verify
echo "[6/6] Waiting for login to complete..."
sleep 3

# Take screenshot for verification
agent-mobile screenshot -o /tmp/login_result.png
echo "Screenshot saved to /tmp/login_result.png"

# Optional: Check for success indicator
agent-mobile snapshot -i > /tmp/post_login_screen.txt

if grep -qi "logout\|home\|dashboard" /tmp/post_login_screen.txt; then
  echo ""
  echo "✓ Login successful!"
  exit 0
else
  echo ""
  echo "⚠ Login may have failed. Please check /tmp/login_result.png"
  exit 1
fi
