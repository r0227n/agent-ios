#!/bin/bash
#
# new-command.sh - Generate CLI command template
#
# Usage: ./scripts/new-command.sh <command-name>
#
# Generates:
# - src/core/<command-name>.rs (command implementation)
# - tests/cli/<command-name>_integration.rs (integration test)
# - Updates src/mod.rs (adds mod, Commands enum, match branch)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Navigate to project root (.claude/skills/agent-mobile-dev/scripts -> ../..)
SKILL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
# Project root is 3 levels up from skill dir (.claude/skills/agent-mobile-dev -> ../../..)
PROJECT_DIR="$(cd "$SKILL_DIR/../../.." && pwd)"

# Check arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <command-name> [description]"
    echo ""
    echo "Example:"
    echo "  $0 vibrate"
    echo "  $0 vibrate \"Vibrate device\""
    exit 1
fi

COMMAND_NAME="$1"
DESCRIPTION="${2:-}"

# Convert command-name to snake_case and PascalCase
SNAKE_CASE=$(echo "$COMMAND_NAME" | tr '-' '_' | tr '[:upper:]' '[:lower:]')
PASCAL_CASE=$(echo "$SNAKE_CASE" | awk -F'_' '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) tolower(substr($i,2))} 1' OFS='')

# Interactive description input if not provided
if [ -z "$DESCRIPTION" ]; then
    echo -n "Description (optional): "
    read DESCRIPTION
fi

# Set default description if empty
if [ -z "$DESCRIPTION" ]; then
    DESCRIPTION="$PASCAL_CASE command"
fi

echo "Generating command: $SNAKE_CASE ($PASCAL_CASE)"
echo "Description: $DESCRIPTION"
echo ""

# File paths
COMMAND_FILE="$PROJECT_DIR/src/core/${SNAKE_CASE}.rs"
TEST_FILE="$PROJECT_DIR/tests/cli/${SNAKE_CASE}_integration.rs"
MOD_FILE="$PROJECT_DIR/src/mod.rs"
TEMPLATE_DIR="$SCRIPT_DIR/../assets"

# Check if files already exist
if [ -f "$COMMAND_FILE" ]; then
    echo "Error: $COMMAND_FILE already exists"
    exit 1
fi

if [ -f "$TEST_FILE" ]; then
    echo "Error: $TEST_FILE already exists"
    exit 1
fi

# Generate command implementation
cat "$TEMPLATE_DIR/command-template.rs" | \
    sed "s/{COMMAND_NAME}/$SNAKE_CASE/g" | \
    sed "s/{CommandName}/$PASCAL_CASE/g" | \
    sed "s/{DESCRIPTION}/$DESCRIPTION/g" \
    > "$COMMAND_FILE"

echo "✓ Created $COMMAND_FILE"

# Generate integration test
cat "$TEMPLATE_DIR/test-template.rs" | \
    sed "s/{COMMAND_NAME}/$SNAKE_CASE/g" | \
    sed "s/{CommandName}/$PASCAL_CASE/g" | \
    sed "s/{DESCRIPTION}/$DESCRIPTION/g" \
    > "$TEST_FILE"

echo "✓ Created $TEST_FILE"

# Update src/mod.rs
# 1. Add mod declaration
if ! grep -q "pub mod $SNAKE_CASE;" "$MOD_FILE"; then
    # Find the line with "pub mod core;" and add after it
    sed -i.bak "/pub mod core;/a\\
pub mod $SNAKE_CASE;" "$MOD_FILE"
    rm "${MOD_FILE}.bak"
    echo "✓ Added 'pub mod $SNAKE_CASE;' to src/mod.rs"
fi

# 2. Add Commands enum variant
if ! grep -q "${PASCAL_CASE}(" "$MOD_FILE"; then
    # Find Commands enum and add variant
    # This is a simple append - manual adjustment may be needed for proper placement
    echo "⚠ Please manually add '${PASCAL_CASE}(${SNAKE_CASE}::${PASCAL_CASE}Args),' to Commands enum in src/mod.rs"
fi

# 3. Add match branch
if ! grep -q "Commands::${PASCAL_CASE}" "$MOD_FILE"; then
    echo "⚠ Please manually add 'Commands::${PASCAL_CASE}(args) => ${SNAKE_CASE}::run(args).await,' to match in src/mod.rs"
fi

echo ""
echo "Next steps:"
echo "  1. cargo build"
echo "  2. Customize ${COMMAND_FILE} implementation"
echo "  3. cargo test --test cli ${SNAKE_CASE}"
echo "  4. /mobile-e2e ios  # Real device verification (REQUIRED!)"
echo ""
echo "Manual updates needed:"
echo "  - Add '${PASCAL_CASE}(${SNAKE_CASE}::${PASCAL_CASE}Args),' to Commands enum in src/mod.rs"
echo "  - Add 'Commands::${PASCAL_CASE}(args) => ${SNAKE_CASE}::run(args).await,' to match in src/mod.rs"
