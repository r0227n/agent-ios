#!/bin/bash
# Complete MV command implementation
# Run this script to finish the implementation

set -e

cd "$(dirname "$0")"

echo "================================"
echo "MV Command Implementation"
echo "================================"
echo ""

# Step 1: Check if routing needs to be added
if grep -q "IdbCommands::Mv" src/main.rs; then
    echo "✅ Mv routing already present in main.rs"
else
    echo "📝 Adding Mv routing to main.rs..."

    # Create backup
    cp src/main.rs src/main.rs.backup

    # Use Python to add the routing
    python3 add_routing.py

    echo "✅ Mv routing added to main.rs"
fi

echo ""
echo "================================"
echo "Building project..."
echo "================================"
cargo build

echo ""
echo "================================"
echo "Build successful! ✅"
echo "================================"
echo ""
echo "Test the command:"
echo "  ./target/debug/agent-mobile idb mv --help"
echo ""
echo "Example usage:"
echo "  ./target/debug/agent-mobile idb mv /tmp/test1 /tmp/test2 --root --udid <YOUR_UDID>"
echo ""
echo "================================"
echo "Ready to commit and push!"
echo "================================"
echo ""
echo "Run these commands:"
echo "  git add ."
echo "  git commit -m \"feat: implement idb file mv command\""
echo "  git push -u origin feature/idb-file-mv"
echo "  gh pr create"
echo ""
