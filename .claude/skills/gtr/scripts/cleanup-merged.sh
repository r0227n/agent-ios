#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: cleanup-merged.sh [--dry-run] [--force]

Remove worktrees for branches that have been merged to main.

Options:
  --dry-run  Show what would be removed without removing
  --force    Skip confirmation prompts
  -h, --help Show this help
USAGE
}

dry_run=false
force=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run|-n) dry_run=true; shift ;;
    --force|-f)   force=true; shift ;;
    -h|--help)    usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

if ! command -v git >/dev/null 2>&1; then
  echo "git not found in PATH" >&2
  exit 1
fi

# Check if gtr is available
if ! git gtr --help >/dev/null 2>&1; then
  echo "git-gtr not found. Install from: https://github.com/coderabbitai/git-worktree-runner" >&2
  exit 1
fi

# Get main branch name
main_branch=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")

# Fetch latest from remote
echo "Fetching latest from origin/$main_branch..."
git fetch origin "$main_branch" --quiet 2>/dev/null || true

# Get merged branches
merged_branches=$(git branch --merged "origin/$main_branch" 2>/dev/null | grep -v "^\*" | grep -v "$main_branch" | grep -v "master" | tr -d ' ' || true)

if [[ -z "$merged_branches" ]]; then
  echo "No merged branches found"
  exit 0
fi

# Get main worktree path to exclude it
main_worktree=$(git worktree list --porcelain | head -1 | sed 's/^worktree //')

# Find worktrees for merged branches
to_remove=()
while IFS= read -r branch; do
  [[ -z "$branch" ]] && continue
  worktree_info=$(git worktree list | grep "\[$branch\]" || true)
  if [[ -n "$worktree_info" ]]; then
    path=$(echo "$worktree_info" | awk '{print $1}')
    # Skip main worktree
    [[ "$path" == "$main_worktree" ]] && continue
    to_remove+=("$branch")
  fi
done <<< "$merged_branches"

if [[ ${#to_remove[@]} -eq 0 ]]; then
  echo "No worktrees to remove"
  exit 0
fi

echo "Worktrees with merged branches:"
for branch in "${to_remove[@]}"; do
  path=$(git worktree list | grep "\[$branch\]" | awk '{print $1}')
  echo "  - $branch ($path)"
done

if [[ "$dry_run" == true ]]; then
  echo ""
  echo "(dry-run mode - no changes made)"
  exit 0
fi

if [[ "$force" != true ]]; then
  read -p "Remove these worktrees? [y/N] " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted"
    exit 0
  fi
fi

# Remove worktrees
for branch in "${to_remove[@]}"; do
  echo "Removing $branch..."
  git gtr rm "$branch" --yes 2>/dev/null || git worktree remove --force "$(git gtr go "$branch" 2>/dev/null)" 2>/dev/null || true
done

echo "Done. Removed ${#to_remove[@]} worktree(s)"
