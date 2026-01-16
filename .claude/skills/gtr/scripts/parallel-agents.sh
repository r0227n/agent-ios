#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: parallel-agents.sh <task-file> [--socket <path>] [--agent <cmd>]

Launch multiple AI agents from a task file (one task per line).

Options:
  --socket <path>  tmux socket path (default: $TMPDIR/gtr-agents.sock)
  --agent <cmd>    AI agent command (default: claude)
  --from <branch>  Base branch for worktrees (default: main)
  -h, --help       Show this help

Task file format (one task per line):
  Fix the authentication bug in auth.rs
  Add unit tests for grpc module
  Refactor error handling in client.rs
USAGE
}

task_file=""
socket="${TMPDIR:-/tmp}/gtr-agents.sock"
agent_cmd="claude"
base_branch="main"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --socket|-s) socket="${2-}"; shift 2 ;;
    --agent|-a)  agent_cmd="${2-}"; shift 2 ;;
    --from|-f)   base_branch="${2-}"; shift 2 ;;
    -h|--help)   usage; exit 0 ;;
    -*) echo "Unknown option: $1" >&2; usage; exit 1 ;;
    *)
      if [[ -z "$task_file" ]]; then
        task_file="$1"; shift
      else
        echo "Unexpected argument: $1" >&2; usage; exit 1
      fi
      ;;
  esac
done

if [[ -z "$task_file" ]]; then
  echo "Task file is required" >&2
  usage
  exit 1
fi

if [[ ! -f "$task_file" ]]; then
  echo "Task file not found: $task_file" >&2
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "git not found in PATH" >&2
  exit 1
fi

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is required but not found in PATH" >&2
  exit 1
fi

# Check if gtr is available
if ! git gtr --help >/dev/null 2>&1; then
  echo "git-gtr not found. Install from: https://github.com/coderabbitai/git-worktree-runner" >&2
  exit 1
fi

# Read tasks (skip empty lines and comments)
tasks=()
while IFS= read -r line || [[ -n "$line" ]]; do
  # Skip empty lines and comments
  [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
  tasks+=("$line")
done < "$task_file"

if [[ ${#tasks[@]} -eq 0 ]]; then
  echo "No tasks found in $task_file" >&2
  exit 1
fi

echo "Launching ${#tasks[@]} agent(s)..."
echo "Socket: $socket"
echo "Agent:  $agent_cmd"
echo "Base:   $base_branch"
echo ""

# Ensure socket directory exists
socket_dir=$(dirname "$socket")
mkdir -p "$socket_dir"

# Create worktrees and launch agents
for i in "${!tasks[@]}"; do
  task="${tasks[$i]}"
  branch="gtr-agent-$i"

  echo "[$i] Creating worktree: $branch"

  # Create worktree (ignore if exists)
  if ! git worktree list | grep -q "\[$branch\]"; then
    git gtr new "$branch" --from "$base_branch" 2>/dev/null || {
      echo "    Warning: Failed to create worktree, trying to use existing branch" >&2
      git worktree add -B "$branch" "../${branch}" "$base_branch" 2>/dev/null || true
    }
  fi

  worktree=$(git gtr go "$branch" 2>/dev/null || git worktree list | grep "\[$branch\]" | awk '{print $1}')

  if [[ -z "$worktree" || ! -d "$worktree" ]]; then
    echo "    Error: Failed to get worktree path for $branch" >&2
    continue
  fi

  # Create tmux session (ignore if exists)
  tmux -S "$socket" new-session -d -s "$branch" 2>/dev/null || true

  # Launch agent
  tmux -S "$socket" send-keys -t "$branch" "cd '$worktree' && $agent_cmd '$task'" Enter

  echo "    Path: $worktree"
  echo "    Task: $task"
  echo ""
done

echo "All agents launched."
echo ""
echo "To monitor all agents:"
echo "  tmux -S '$socket' attach"
echo ""
echo "To check individual agent:"
for i in "${!tasks[@]}"; do
  echo "  tmux -S '$socket' capture-pane -p -t gtr-agent-$i -S -200"
done
echo ""
echo "To kill all agents:"
echo "  tmux -S '$socket' kill-server"
