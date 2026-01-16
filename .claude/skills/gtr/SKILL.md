---
name: gtr
description: Manage git worktrees for parallel development workflows with AI agents.
metadata: {"clawdbot":{"emoji":"🌳","os":["darwin","linux"],"requires":{"bins":["git","git-gtr"]}}}
---

# gtr Skill (Clawdbot)

Use gtr for parallel development workflows. Each worktree is an isolated copy of your repo where you can work on a different branch simultaneously.

## Quickstart

```bash
# Create a new worktree for a feature branch
git gtr new feature-auth --from main

# List all worktrees
git gtr list

# Run a command in a worktree
git gtr run feature-auth "cargo test"

# Open worktree in editor
git gtr editor feature-auth

# Clean up merged worktrees
git gtr clean --merged
```

After creating worktrees, you can monitor them with:

```bash
# List all worktrees
{baseDir}/scripts/list-worktrees.sh

# Check status of a specific worktree
{baseDir}/scripts/worktree-status.sh <branch>
```

## Core Commands Reference

### Creating Worktrees

```bash
# Basic creation (creates branch from current HEAD)
git gtr new <branch-name>

# Create from specific base branch
git gtr new <branch-name> --from main

# Create from current branch
git gtr new <branch-name> --from-current

# Create and open in editor
git gtr new <branch-name> --editor
git gtr new <branch-name> -e

# Create and launch AI agent
git gtr new <branch-name> --ai
git gtr new <branch-name> -a

# Create with both editor and AI
git gtr new <branch-name> -e -a

# Force create (same branch in multiple worktrees)
git gtr new <branch-name> --force --name variant-1

# Skip file copy
git gtr new <branch-name> --no-copy

# Skip git fetch
git gtr new <branch-name> --no-fetch
```

### Managing Worktrees

```bash
# List all worktrees
git gtr list

# Get path to a worktree (for cd, scripts, etc.)
git gtr go <branch-name>

# Run command in worktree context
git gtr run <branch-name> "<command>"
git gtr run feature-auth "npm test"
git gtr run feature-auth "cargo build --release"

# Copy files to a worktree
git gtr copy <target-branch>
git gtr copy <target-branch> --from <source-branch>
git gtr copy --all  # copy to all worktrees

# Remove a worktree
git gtr rm <branch-name>
git gtr rm <branch-name> --delete-branch  # also delete branch
git gtr rm <branch-name> --force          # force removal
```

### Opening Tools

```bash
# Open in configured editor (cursor, vscode, zed, etc.)
git gtr editor <branch-name>
git gtr editor <branch-name> --editor cursor

# Launch AI coding agent (claude, codex, copilot, aider, etc.)
git gtr ai <branch-name>
git gtr ai <branch-name> --ai claude
git gtr ai <branch-name> -- --yolo  # pass args to agent
```

### Configuration

```bash
# View current config
git gtr config

# Set default editor
git gtr config set gtr.editor.default cursor

# Set default AI agent
git gtr config set gtr.ai.default claude

# Add files to copy on worktree creation
git gtr config add gtr.copy.include "**/.env.example"
git gtr config add gtr.copy.include "**/.env.local"

# Add post-create hook
git gtr config add gtr.hook.postCreate "npm install"
```

### Cleanup

```bash
# Remove worktrees for merged branches
git gtr clean --merged

# Dry run to see what would be cleaned
git gtr clean --merged --dry-run

# Skip confirmation
git gtr clean --merged --yes

# Check gtr health
git gtr doctor
```

## Parallel Development Patterns

### Pattern 1: Multiple Feature Branches

Work on multiple features simultaneously without stashing or switching branches:

```bash
# Create worktrees for each feature
git gtr new feature-auth --from main
git gtr new feature-ui --from main
git gtr new bugfix-123 --from main

# Run tests in all worktrees
for branch in feature-auth feature-ui bugfix-123; do
  echo "Testing $branch..."
  git gtr run "$branch" "cargo test" &
done
wait
```

### Pattern 2: PR Review Workflow

Create isolated worktree for reviewing pull requests:

```bash
# Fetch and create worktree for PR branch
git fetch origin pull/123/head:pr-123
git gtr new pr-123 --from pr-123

# Review in editor
git gtr editor pr-123

# Run tests
git gtr run pr-123 "cargo test"

# Clean up after review
git gtr rm pr-123 --delete-branch
```

### Pattern 3: Parallel AI Agents with tmux

Combine gtr with tmux skill to run multiple AI agents in parallel:

```bash
SOCKET="${TMPDIR:-/tmp}/gtr-agents.sock"

# Create worktrees for parallel tasks
git gtr new fix-bug-1 --from main
git gtr new fix-bug-2 --from main
git gtr new add-tests --from main

# Create tmux sessions for each worktree
for branch in fix-bug-1 fix-bug-2 add-tests; do
  worktree_path="$(git gtr go "$branch")"
  tmux -S "$SOCKET" new-session -d -s "$branch"
  tmux -S "$SOCKET" send-keys -t "$branch" "cd '$worktree_path'" Enter
done

# Launch agents with different tasks
tmux -S "$SOCKET" send-keys -t fix-bug-1 "claude 'Fix the authentication bug in auth.rs'" Enter
tmux -S "$SOCKET" send-keys -t fix-bug-2 "claude 'Fix the database connection timeout'" Enter
tmux -S "$SOCKET" send-keys -t add-tests "claude 'Add unit tests for the grpc module'" Enter

# Monitor progress
echo "To monitor: tmux -S '$SOCKET' attach"
```

### Pattern 4: Batch Testing

Run the same test suite across multiple branches to verify changes:

```bash
# List worktrees and run tests
{baseDir}/scripts/list-worktrees.sh | while IFS=$'\t' read -r branch path; do
  echo "Testing $branch..."
  git gtr run "$branch" "cargo test" 2>&1 | tail -5
done
```

## Integration with tmux Skill

For long-running AI agents, combine gtr with the tmux skill:

1. **Create isolated worktrees** - Each agent gets its own worktree to avoid conflicts
2. **Launch agents in tmux sessions** - Use tmux for interactive agent management
3. **Monitor and collect results** - Use tmux capture-pane to check progress

Example orchestration:

```bash
# Setup
SOCKET="${TMPDIR:-/tmp}/parallel-agents.sock"
TASKS=("Fix bug in auth module" "Add logging to grpc client" "Refactor error handling")

for i in "${!TASKS[@]}"; do
  branch="agent-task-$i"

  # Create worktree (ignore if exists)
  git gtr new "$branch" --from main 2>/dev/null || true
  worktree="$(git gtr go "$branch")"

  # Create tmux session and launch agent
  tmux -S "$SOCKET" new-session -d -s "$branch"
  tmux -S "$SOCKET" send-keys -t "$branch" "cd '$worktree' && claude '${TASKS[$i]}'" Enter
done

echo "Launched ${#TASKS[@]} agents. Monitor with: tmux -S '$SOCKET' attach"
```

Use `wait-for-text.sh` from the tmux skill to detect agent completion:

```bash
# Wait for shell prompt indicating agent completion
{tmuxSkillDir}/scripts/wait-for-text.sh -t agent-task-0:0.0 -p '^\$ ' -T 300
```

## Helper Scripts

### list-worktrees.sh

`{baseDir}/scripts/list-worktrees.sh [options]`

List all gtr-managed worktrees with their status.

```bash
# Basic list (branch and path, tab-separated)
{baseDir}/scripts/list-worktrees.sh

# Include git status
{baseDir}/scripts/list-worktrees.sh --status

# JSON output
{baseDir}/scripts/list-worktrees.sh --json

# Only show merged worktrees
{baseDir}/scripts/list-worktrees.sh --merged
```

Options:
- `-j, --json` Output as JSON
- `-s, --status` Include git status for each worktree
- `-m, --merged` Only show worktrees with merged branches

### create-worktree.sh

`{baseDir}/scripts/create-worktree.sh <branch> [--from <base>] [--no-fail]`

Create a worktree with error handling and validation.

```bash
# Basic creation
{baseDir}/scripts/create-worktree.sh feature-auth

# Create from specific branch
{baseDir}/scripts/create-worktree.sh feature-auth --from main

# Don't fail if worktree exists
{baseDir}/scripts/create-worktree.sh feature-auth --no-fail
```

Options:
- `--from <base>` Base branch to create from
- `--no-fail` Don't exit with error if worktree already exists

### cleanup-merged.sh

`{baseDir}/scripts/cleanup-merged.sh [--dry-run] [--force]`

Remove worktrees for branches that have been merged.

```bash
# Preview what would be removed
{baseDir}/scripts/cleanup-merged.sh --dry-run

# Remove without confirmation
{baseDir}/scripts/cleanup-merged.sh --force
```

Options:
- `--dry-run` Show what would be removed without removing
- `--force` Skip confirmation prompts

### worktree-status.sh

`{baseDir}/scripts/worktree-status.sh <branch>`

Get detailed status of a specific worktree.

```bash
{baseDir}/scripts/worktree-status.sh feature-auth
```

Output includes:
- Path
- Current commit
- Uncommitted changes
- Ahead/behind status relative to main

### parallel-agents.sh

`{baseDir}/scripts/parallel-agents.sh <task-file> [options]`

Launch multiple AI agents from a task file (one task per line).

```bash
# Create task file
cat > /tmp/tasks.txt << 'EOF'
Fix the authentication bug in auth.rs
Add unit tests for the grpc module
Refactor error handling in client.rs
EOF

# Launch agents
{baseDir}/scripts/parallel-agents.sh /tmp/tasks.txt

# With custom options
{baseDir}/scripts/parallel-agents.sh /tmp/tasks.txt --agent codex --from develop
```

Options:
- `--socket <path>` tmux socket path (default: $TMPDIR/gtr-agents.sock)
- `--agent <cmd>` AI agent command (default: claude)
- `--from <branch>` Base branch for worktrees (default: main)

## Best Practices

1. **Use descriptive branch names** - Include ticket numbers or feature names
2. **Base from main/master** - Avoid divergent histories by branching from the main branch
3. **Clean up regularly** - Run `git gtr clean --merged` after PRs are merged
4. **One agent per worktree** - Never run multiple agents in the same worktree
5. **Check status before cleanup** - Use `--dry-run` to verify what will be removed
6. **Use separate git worktrees for parallel fixes** - No branch conflicts

## Troubleshooting

### Worktree already exists

```bash
# Check existing worktrees
git gtr list

# Remove and recreate
git gtr rm <branch>
git gtr new <branch> --from main
```

### Branch conflicts

```bash
# Delete local branch before creating worktree
git branch -D <branch>
git gtr new <branch> --from main
```

### Stale worktrees

```bash
# Prune worktrees with missing directories
git worktree prune
```

### gtr not found

```bash
# Install gtr
git clone https://github.com/coderabbitai/git-worktree-runner.git
cd git-worktree-runner
./install.sh
```

### tmux socket permission issues

```bash
# Use a socket in a directory you own
SOCKET="${TMPDIR:-/tmp}/gtr-agents.sock"
# Or
SOCKET="$HOME/.local/share/gtr/agents.sock"
mkdir -p "$(dirname "$SOCKET")"
```
