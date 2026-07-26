---
name: finding-uncommitted-work
disable-model-invocation: true
description: |
  Systematic scan for work-in-progress across repositories. Use at end of
  day/week, before context switches, when resuming after a break, or before
  machine migrations. Finds uncommitted changes, unpushed commits, unmerged
  branches, and non-main branch state.
---

# Finding Uncommitted Work

Systematic scan for work-in-progress across repositories that hasn't been committed, pushed, or merged.

Searches recursively for git repos. Excludes process artifacts (DATA_SOURCES.md, LEARNINGS.md, PLAN_*.md, etc.). Finds: uncommitted changes, unpushed commits, unmerged branches, repos on non-main branches.

Triage priority: uncommitted changes (highest risk), unpushed commits (local only), unmerged branches (safe on remote), non-main branches (context indicator).

## Common Mistakes

1. **Scanning too deep** — `find ~/repo -maxdepth 10` finds nested .git dirs inside vendor/node_modules. Use maxdepth 3-4.
2. **Missing stashes** — `git stash list` is easy to skip but stashes contain real work that won't show in status.
3. **Ignoring worktrees** — `git worktree list` may reveal isolated copies with uncommitted work.
4. **Counting process artifacts as real changes** — DATA_SOURCES.md, LEARNINGS.md, PLAN_*.md are expected to be dirty. Exclude them from "uncommitted work" alerts.
5. **Not checking submodules** — submodules can have their own uncommitted state that parent `git status` doesn't show.
