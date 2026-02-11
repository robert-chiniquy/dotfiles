---
name: finding-uncommitted-work
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
