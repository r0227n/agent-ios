#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: list-worktrees.sh [-j|-s|-m]

List git worktrees managed by gtr.

Options:
  -j, --json     Output as JSON
  -s, --status   Include git status for each worktree
  -m, --merged   Only show worktrees with merged branches
  -h, --help     Show this help
USAGE
}

json_output=false
include_status=false
only_merged=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    -j|--json)   json_output=true; shift ;;
    -s|--status) include_status=true; shift ;;
    -m|--merged) only_merged=true; shift ;;
    -h|--help)   usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

if ! command -v git >/dev/null 2>&1; then
  echo "git not found in PATH" >&2
  exit 1
fi

# Get main branch name
main_branch=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")

# Get the main worktree path to exclude it
main_worktree=$(git worktree list --porcelain | head -1 | sed 's/^worktree //')

# Parse git worktree list output
git worktree list --porcelain | awk '
  /^worktree / { path = substr($0, 10) }
  /^branch /   { branch = substr($0, 8); gsub("refs/heads/", "", branch) }
  /^$/ {
    if (branch != "" && path != "") {
      print branch "\t" path
    }
    path = ""; branch = ""
  }
  END {
    if (branch != "" && path != "") {
      print branch "\t" path
    }
  }
' | while IFS=$'\t' read -r branch path; do
  # Skip main worktree
  [[ "$path" == "$main_worktree" ]] && continue

  # Check if merged
  if [[ "$only_merged" == true ]]; then
    if ! git branch --merged "origin/$main_branch" 2>/dev/null | grep -q "^[[:space:]]*$branch$"; then
      continue
    fi
  fi

  if [[ "$json_output" == true ]]; then
    status="0"
    if [[ "$include_status" == true && -d "$path" ]]; then
      status=$(cd "$path" && git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
    fi
    printf '{"branch":"%s","path":"%s","changes":%s}\n' "$branch" "$path" "$status"
  else
    if [[ "$include_status" == true ]]; then
      if [[ -d "$path" ]]; then
        changes=$(cd "$path" && git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
        printf '%s\t%s\t(%s changes)\n' "$branch" "$path" "$changes"
      else
        printf '%s\t%s\t(path missing)\n' "$branch" "$path"
      fi
    else
      printf '%s\t%s\n' "$branch" "$path"
    fi
  fi
done
