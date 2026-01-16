#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: worktree-status.sh <branch>

Get detailed status of a specific worktree.

Options:
  -h, --help  Show this help
USAGE
}

branch=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    -*) echo "Unknown option: $1" >&2; usage; exit 1 ;;
    *)
      if [[ -z "$branch" ]]; then
        branch="$1"; shift
      else
        echo "Unexpected argument: $1" >&2; usage; exit 1
      fi
      ;;
  esac
done

if [[ -z "$branch" ]]; then
  echo "Branch name is required" >&2
  usage
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "git not found in PATH" >&2
  exit 1
fi

# Get worktree path
path=""
if command -v git-gtr >/dev/null 2>&1 || git gtr --help >/dev/null 2>&1; then
  path=$(git gtr go "$branch" 2>/dev/null || true)
fi

# Fallback: parse worktree list
if [[ -z "$path" ]]; then
  worktree_info=$(git worktree list | grep "\[$branch\]" || true)
  if [[ -n "$worktree_info" ]]; then
    path=$(echo "$worktree_info" | awk '{print $1}')
  fi
fi

if [[ -z "$path" || ! -d "$path" ]]; then
  echo "Worktree '$branch' not found" >&2
  exit 1
fi

echo "Branch:     $branch"
echo "Path:       $path"

# Get current commit
commit=$(cd "$path" && git rev-parse --short HEAD 2>/dev/null || echo "unknown")
commit_msg=$(cd "$path" && git log -1 --format='%s' 2>/dev/null || echo "")
echo "Commit:     $commit - $commit_msg"

# Get main branch name
main_branch=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")

# Get ahead/behind status
ahead_behind=$(cd "$path" && git rev-list --left-right --count "origin/$main_branch...$branch" 2>/dev/null || echo "0	0")
behind=$(echo "$ahead_behind" | awk '{print $1}')
ahead=$(echo "$ahead_behind" | awk '{print $2}')
echo "Status:     $ahead ahead, $behind behind origin/$main_branch"

# Check if merged
if git branch --merged "origin/$main_branch" 2>/dev/null | grep -q "^[[:space:]]*$branch$"; then
  echo "Merged:     Yes (safe to remove)"
else
  echo "Merged:     No"
fi

# Get uncommitted changes
changes=$(cd "$path" && git status --porcelain 2>/dev/null || true)
if [[ -n "$changes" ]]; then
  change_count=$(echo "$changes" | wc -l | tr -d ' ')
  echo "Changes:    $change_count uncommitted file(s)"
  echo "$changes" | head -10 | sed 's/^/            /'
  if [[ $change_count -gt 10 ]]; then
    echo "            ... and $((change_count - 10)) more"
  fi
else
  echo "Changes:    Working tree clean"
fi
