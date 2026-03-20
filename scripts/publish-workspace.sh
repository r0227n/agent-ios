#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/publish-workspace.sh [--dry-run]

Publishes the workspace crates in dependency order so the root `agent-mobile`
package can resolve its sibling crates from crates.io.

Environment:
  CARGO_PUBLISH_WAIT_SECONDS   Seconds to wait between publishes (default: 30)
EOF
}

dry_run_args=()
case "${1:-}" in
  "")
    ;;
  --dry-run)
    dry_run_args=(--dry-run)
    ;;
  -h|--help)
    usage
    exit 0
    ;;
  *)
    usage
    exit 1
    ;;
esac

cd "$(dirname "$0")/.."

packages=(
  agent-mobile-core
  agent-mobile-platform-ios
  agent-mobile-platform-android
  agent-mobile-gateway
  agent-mobile
)

wait_seconds="${CARGO_PUBLISH_WAIT_SECONDS:-30}"
last_index=$((${#packages[@]} - 1))

for index in "${!packages[@]}"; do
  package="${packages[$index]}"
  echo "Publishing ${package}..."
  cargo publish -p "${package}" "${dry_run_args[@]}"

  if [[ ${#dry_run_args[@]} -eq 0 && "${index}" -lt "${last_index}" ]]; then
    echo "Waiting ${wait_seconds}s for crates.io index propagation..."
    sleep "${wait_seconds}"
  fi
done
