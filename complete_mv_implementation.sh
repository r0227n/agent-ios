#!/bin/bash
# Script to complete the mv command implementation

set -e

echo "Step 1: Adding routing to main.rs..."

# Find the last IdbCommands match arm and add Mv routing after it
# This needs to be done manually by finding the right location

MAIN_FILE="src/main.rs"

# Backup main.rs
cp "$MAIN_FILE" "${MAIN_FILE}.backup"

# Create a temporary file with the routing code
cat > /tmp/mv_routing.txt << 'EOF'
            IdbCommands::Mv {
                src_paths,
                dst_path,
                bundle_id,
                root,
                udid,
            } => {
                cli::idb::mv::run(src_paths, dst_path, bundle_id, root, udid).await?;
            }
EOF

echo "Add the following routing code to src/main.rs after the Uninstall match arm:"
cat /tmp/mv_routing.txt
echo ""
echo "Press Enter after you've added the routing code..."
read

echo "Step 2: Building the project..."
cargo build

echo "Step 3: Testing the implementation..."
echo "Run: ./target/debug/agent-mobile idb mv /tmp/test1 /tmp/test2 --root --udid <YOUR_UDID>"
