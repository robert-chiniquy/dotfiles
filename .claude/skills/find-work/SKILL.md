---
name: find-work
disable-model-invocation: true
description: |
  Find uncommitted changes, unpushed commits, unmerged branches, and
  incomplete code markers (TODO, FIXME, HACK) across repositories.
  Use at end of day, before context switches, or to audit work in progress.
---

# Find Work

Two modes: git state and code markers.

## Git State Scan

For each repo (or all repos under ~/repo/):

1. **Uncommitted changes**: `git status --porcelain`
2. **Unpushed commits**: `git log @{u}..HEAD --oneline`
3. **Stashed work**: `git stash list`
4. **Non-main branches**: `git branch` — anything not on main
5. **Unmerged branches**: `git branch --no-merged main`

Report format: one line per finding, grouped by repo.

## Code Marker Scan

Search for incomplete work markers in the codebase:

* `TODO`, `FIXME`, `TBD`, `XXX`, `HACK`
* `stub`, `placeholder`, `not implemented`
* Functions that just `return nil` or `panic("not implemented")`

For each marker found: file:line, surrounding context, severity assessment.

## Common Mistakes

1. Scanning node_modules, vendor, or generated directories
2. Missing stashed work (always check `git stash list`)
3. Not checking worktrees for orphaned work
4. Reporting TODOs in vendored/third-party code
5. Missing unpushed tags
