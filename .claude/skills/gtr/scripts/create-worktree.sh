#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: create-worktree.sh <branch> [--from <base>] [--no-fail]

Create a git worktree with error handling.

Options:
  --from <base>  Base branch to create from (default: current branch)
  --no-fail      Don't exit with error if worktree exists
  -h, --help     Show this help
USAGE
}

branch=""
base_branch=""
no_fail=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --from)    base_branch="${2-}"; shift 2 ;;
    --no-fail) no_fail=true; shift ;;
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

# Check if gtr is available
if ! git gtr --help >/dev/null 2>&1; then
  echo "git-gtr not found. Install from: https://github.com/coderabbitai/git-worktree-runner" >&2
  exit 1
fi

# Check if worktree already exists
if git worktree list | grep -q "\[$branch\]"; then
  if [[ "$no_fail" == true ]]; then
    echo "Worktree for '$branch' already exists" >&2
    git gtr go "$branch"
    exit 0
  else
    echo "Error: Worktree for '$branch' already exists" >&2
    echo "Use 'git gtr rm $branch' to remove the existing worktree first" >&2
    exit 1
  fi
fi

# Check if branch already exists (and isn't a worktree)
if git show-ref --verify --quiet "refs/heads/$branch" 2>/dev/null; then
  echo "Warning: Branch '$branch' already exists, will use existing branch" >&2
fi

# Create worktree
if [[ -n "$base_branch" ]]; then
  git gtr new "$branch" --from "$base_branch"
else
  git gtr new "$branch"
fi

# Output the path
git gtr go "$branch"
