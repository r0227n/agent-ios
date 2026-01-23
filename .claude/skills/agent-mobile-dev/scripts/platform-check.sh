#!/bin/bash
#
# platform-check.sh - iOS platform implementation decision helper
#
# Usage: ./scripts/platform-check.sh <feature-name>
#
# Checks proto/idb.proto for RPC definitions and recommends implementation approach.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Navigate to project root (.claude/skills/agent-mobile-dev/scripts -> ../..)
SKILL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
# Project root is 3 levels up from skill dir (.claude/skills/agent-mobile-dev -> ../../..)
PROJECT_DIR="$(cd "$SKILL_DIR/../../.." && pwd)"
PROTO_FILE="$PROJECT_DIR/proto/idb.proto"

# Check arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <feature-name>"
    echo ""
    echo "Example:"
    echo "  $0 accessibility"
    echo "  $0 clipboard"
    exit 1
fi

FEATURE="$1"

echo "Checking proto/idb.proto for: $FEATURE"
echo ""

# Search for RPC definitions (case-insensitive)
RPC_MATCHES=$(grep -i "rpc.*$FEATURE" "$PROTO_FILE" || true)

if [ -n "$RPC_MATCHES" ]; then
    echo "✓ Found RPC definition(s):"
    echo "$RPC_MATCHES" | sed 's/^/  /'
    echo ""
    echo "Recommendation: Use idb gRPC"
    echo ""
    echo "Implementation:"
    echo "  - Use with_client() pattern"
    echo "  - Location: src/core/<feature>.rs or src/idb/<feature>.rs"
    echo ""
    echo "Example:"
    echo "  pub async fn run(args: MyArgs) -> CommandResult {"
    echo "      with_client(args.device.udid.as_deref(), |mut client| async move {"
    echo "          // Call gRPC method"
    echo "          client.<method>(...).await?;"
    echo "          Ok(())"
    echo "      }).await"
    echo "  }"
    echo ""
    echo "References:"
    echo "  - proto/idb.proto (RPC definition)"
    echo "  - src/helpers/client.rs (with_client implementation)"
    echo "  - references/platform-decisions.md (detailed guidance)"
else
    echo "✗ No RPC found for: $FEATURE"
    echo ""
    echo "Recommendation: Use xcrun simctl"
    echo ""
    echo "Implementation:"
    echo "  - Use simctl::management module"
    echo "  - Location: crates/platform-ios/src/simctl/management.rs"
    echo ""
    echo "Example (simctl command):"
    echo "  xcrun simctl <subcommand> <udid> [args...]"
    echo ""
    echo "Common simctl commands:"
    echo "  - boot <udid>              # Start simulator"
    echo "  - shutdown <udid>          # Stop simulator"
    echo "  - create <name> <type>     # Create simulator"
    echo "  - delete <udid>            # Delete simulator"
    echo "  - pbcopy <udid>            # Copy to clipboard"
    echo "  - pbpaste <udid>           # Paste from clipboard"
    echo ""
    echo "References:"
    echo "  - crates/platform-ios/src/simctl/management.rs (simctl wrapper)"
    echo "  - references/platform-decisions.md (detailed guidance)"
fi

echo ""
echo "Decision criteria:"
echo "  ✓ RPC in proto        → idb gRPC"
echo "  ✓ Streaming needed    → idb gRPC"
echo "  ✓ Real device support → idb gRPC"
echo "  ✓ Simulator lifecycle → xcrun simctl"
echo "  ✓ No RPC in proto     → xcrun simctl"
echo ""
echo "For detailed decision logic, see: references/platform-decisions.md"
