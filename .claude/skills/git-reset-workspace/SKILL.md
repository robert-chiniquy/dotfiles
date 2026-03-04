---
name: git-reset-workspace
description: |
  Multi-phase workspace cleanup. Stops background agents, inventories
  uncommitted work, switches to main, prunes stale branches, cleans
  worktrees, verifies clean state. Safe by default — never destroys
  work without confirmation. Use for end-of-day cleanup, before context
  switches, or when a repo is in a messy state.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
  - AskUserQuestion
  - Agent
---

# Reset Workspace

Systematic workspace cleanup in six phases. Safe by default — every destructive
action requires confirmation.

## When to Use

- End of day/week cleanup
- Before switching to a different project
- When a repo is in a messy state (stale branches, orphan worktrees)
- Before machine migration
- When resuming after a long break and state is unclear

## Phases

### Phase 1: Stop Background Processes

Check for and stop background processes related to development:

```bash
# Claude Code sessions (show, don't kill — user decides)
ps aux | grep "claude" | grep -v grep

# Background build/test processes
ps aux | grep -E "go test|npm test|make" | grep -v grep

# Dev servers
lsof -i -P | grep LISTEN | grep -v "$(whoami)" --invert-match
```

Present findings. Ask before killing anything.

### Phase 2: Inventory Uncommitted Work

For the current repo (and optionally all repos under ~/repo/):

```bash
# Current repo
git status --short
git stash list
git diff --stat

# All repos (if requested)
find ~/repo -maxdepth 3 -name .git -type d -exec dirname {} \; | while read repo; do
  changes=$(git -C "$repo" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
  stashes=$(git -C "$repo" stash list 2>/dev/null | wc -l | tr -d ' ')
  branch=$(git -C "$repo" branch --show-current 2>/dev/null)
  [ "$changes" -gt 0 ] || [ "$stashes" -gt 0 ] || [ "$branch" != "main" -a "$branch" != "master" ] && \
    echo "$repo: ${changes} changes, ${stashes} stashes, on $branch"
done
```

**Do NOT proceed past this phase if there is uncommitted work.** Present
findings and ask the user what to do:
- Commit it
- Stash it
- Discard it (requires explicit approval)

### Phase 3: Switch to Main

```bash
git checkout main 2>/dev/null || git checkout master
git pull --rebase
```

If checkout fails (uncommitted changes), loop back to Phase 2.

### Phase 4: Prune Stale Branches

List branches with age and merge status:

```bash
git branch --merged main | grep -v "main\|master\|\*"
git branch --no-merged main --format="%(refname:short) %(committerdate:relative)"
```

Present two lists:
1. **Merged branches** (safe to delete — already in main)
2. **Unmerged branches** (may have work — show last commit date and message)

Ask before deleting. For merged branches, suggest batch delete. For unmerged,
require individual confirmation.

```bash
# Delete merged branches (after approval)
git branch --merged main | grep -v "main\|master\|\*" | xargs git branch -d
```

Prune remote tracking branches:
```bash
git remote prune origin
```

### Phase 5: Clean Worktrees

```bash
git worktree list
```

For each worktree that isn't the main working tree:
- Check if it has uncommitted changes
- Check if its branch still exists
- Present findings

```bash
# Remove orphaned worktrees (after approval)
git worktree prune
```

### Phase 6: Verify Clean State

Final verification:

```bash
git status
git branch
git stash list
git worktree list
```

Expected state:
- On main/master
- No uncommitted changes
- No stashes (or intentionally kept stashes)
- Only main + intentionally kept branches
- Only the primary worktree

Present final state. Note any intentional deviations.

## Safety Rules

1. **Never `git clean -fdx`** without showing what will be deleted and getting approval
2. **Never `git reset --hard`** without confirming no work will be lost
3. **Never delete unmerged branches** without individual confirmation
4. **Never kill processes** without showing what they are first
5. **Stale = merged into main or last commit >30 days ago** — not just "old"
6. **Worktrees with changes are not orphaned** — they have work in progress

## Common Mistakes

1. **Deleting a branch that has unpushed commits** — always check `git log main..branch` before deleting
2. **Pruning a worktree that's mid-work** — check `git -C <worktree> status` first
3. **Killing a background process that belongs to another project** — identify the process fully before killing
4. **Running on a shared repo without coordination** — don't prune remote branches other people use
5. **Forgetting stashes** — `git stash list` is easy to skip but stashes contain real work
