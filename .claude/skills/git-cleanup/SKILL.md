---
name: git-cleanup
disable-model-invocation: true
description: |
  Multi-phase workspace cleanup. Inventories uncommitted work, switches to
  main, prunes branches, cleans worktrees. Safe by default. Use for
  end-of-day cleanup or when a repo is messy.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
  - AskUserQuestion
  - Agent
---

# Git Cleanup

Six phases. Each confirms before destructive action.

### Phase 1: Inventory
`git status`, `git stash list`, `git branch -v`. Show what exists.

### Phase 2: Save work
Stash or commit anything uncommitted. Never discard without asking.

### Phase 3: Switch to main
`git checkout main && git pull`. If main is behind, fast-forward.

### Phase 4: Prune branches
List branches merged into main. Confirm before deleting.
Never delete branches with unmerged commits without explicit approval.

### Phase 5: Clean worktrees
`git worktree list`. Remove worktrees that point to deleted branches.

### Phase 6: Verify
`git status` should be clean. `git branch` should be minimal.
Report final state.

## Common Mistakes

1. Deleting a branch that has unpushed commits
2. Pruning a worktree another agent is using
3. Force-cleaning without checking stash first
4. Not pulling main before pruning (branches look unmerged)
5. Cleaning during an active rebase
