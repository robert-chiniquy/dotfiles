# Finding Uncommitted Work

Systematic scan for work-in-progress across repositories that hasn't been committed, pushed, or merged.

## When to Use

- End of day/week to ensure nothing is lost
- Before context switches to capture state
- When resuming work after a break
- Before machine migrations or backups

## Safe Preparatory Operations

Before scanning, `git fetch` or `git pull --rebase origin main` is acceptable if you're on main with no uncommitted changes. This ensures accurate detection of unpushed commits and unmerged branches against current remote state.

**Safe if:**
- On main/master branch
- No uncommitted changes (after exclusions)
- No staged files

**Skip if:**
- On a feature branch (rebase could cause conflicts)
- Has uncommitted work (could be lost in merge conflicts)

The goal is fresh remote state without risking any local artifacts we're trying to preserve.

## Recursive Search

Search recursively for git repos - they may be nested (e.g., `research/spike-classifiers/` contains its own git repo). Use `find` to locate all `.git` directories:

```bash
# Find all git repos under a directory
find "$TARGET_DIR" -name .git -type d 2>/dev/null | while read gitdir; do
  repo_dir=$(dirname "$gitdir")
  # ... scan logic
done
```

Don't assume repos are only at the first level.

## Exclusions

Exclude files produced by the global project process. These are working documents, not deliverables:

```bash
# Process artifacts to exclude
EXCLUDE_PATTERNS=(
  "project.md"
  "DATA_SOURCES.md"
  "LEARNINGS.md"
  "GLOSSARY.md"
  "HUMAN_TODO.md"
  "GAP_ANALYSIS.md"
  "PLAN_*.md"
  "REMAINING_TODOS.md"
  "TODO_*.md"
  "CLAUDE.md"
  "reports/"
)
```

These files are side effects of the work process, not shippable code. When counting uncommitted changes, filter them out to see what actually matters.

## What to Find

### 1. Uncommitted Changes
Files modified but not staged or committed.

```bash
# Exclude pattern for process artifacts
EXCLUDE='project\.md\|DATA_SOURCES\.md\|LEARNINGS\.md\|GLOSSARY\.md\|HUMAN_TODO\.md\|GAP_ANALYSIS\.md\|PLAN_.*\.md\|REMAINING_TODOS\.md\|TODO_.*\.md\|CLAUDE\.md\|reports/'

# Check single repo (excluding process artifacts)
git status --porcelain | grep -v "$EXCLUDE" | wc -l

# Scan directory of repos
for dir in */; do
  if [ -d "$dir/.git" ]; then
    count=$(git -C "$dir" status --porcelain 2>/dev/null | grep -v "$EXCLUDE" | wc -l | tr -d ' ')
    if [ "$count" -gt 0 ]; then
      echo "$dir: $count uncommitted files"
    fi
  fi
done
```

### 2. Unpushed Commits
Commits on local branches not yet pushed to remote.

```bash
# Check single repo
git log --oneline @{u}..HEAD 2>/dev/null | wc -l

# Scan directory of repos
for dir in */; do
  if [ -d "$dir/.git" ]; then
    unpushed=$(git -C "$dir" log --oneline @{u}..HEAD 2>/dev/null | wc -l | tr -d ' ')
    if [ "$unpushed" -gt 0 ]; then
      echo "$dir: $unpushed unpushed commits"
    fi
  fi
done
```

### 3. Unmerged Branches
Local branches not yet merged to main/master.

```bash
# Check single repo (branches not merged to main)
git branch --no-merged main 2>/dev/null | wc -l

# Scan directory of repos
for dir in */; do
  if [ -d "$dir/.git" ]; then
    # Try main first, fall back to master
    main_branch="main"
    git -C "$dir" rev-parse --verify main >/dev/null 2>&1 || main_branch="master"

    unmerged=$(git -C "$dir" branch --no-merged "$main_branch" 2>/dev/null | wc -l | tr -d ' ')
    if [ "$unmerged" -gt 0 ]; then
      branches=$(git -C "$dir" branch --no-merged "$main_branch" 2>/dev/null | sed 's/^..//' | tr '\n' ', ' | sed 's/,$//')
      echo "$dir: $unmerged unmerged branches ($branches)"
    fi
  fi
done
```

### 4. Current Branch Not Main
Repos where HEAD is on a feature branch.

```bash
for dir in */; do
  if [ -d "$dir/.git" ]; then
    branch=$(git -C "$dir" branch --show-current 2>/dev/null)
    if [ -n "$branch" ] && [ "$branch" != "main" ] && [ "$branch" != "master" ]; then
      echo "$dir: on branch $branch"
    fi
  fi
done
```

## Full Scan Script

```bash
#!/bin/bash
# find-uncommitted-work.sh - Scan repos for work in progress (recursive)

TARGET_DIR="${1:-.}"

# Process artifacts to exclude (not shippable, just working docs)
EXCLUDE_GREP='project\.md\|DATA_SOURCES\.md\|LEARNINGS\.md\|GLOSSARY\.md\|HUMAN_TODO\.md\|GAP_ANALYSIS\.md\|PLAN_.*\.md\|REMAINING_TODOS\.md\|TODO_.*\.md\|CLAUDE\.md\|reports/'

echo "=== Scanning $TARGET_DIR recursively for uncommitted work ==="
echo

# Find all git repos recursively
find "$TARGET_DIR" -name .git -type d 2>/dev/null | while read gitdir; do
  dir=$(dirname "$gitdir")
  repo=${dir#$TARGET_DIR/}  # relative path from target
  issues=()

  # Uncommitted changes (excluding process artifacts)
  uncommitted=$(git -C "$dir" status --porcelain 2>/dev/null | grep -v "$EXCLUDE_GREP" | wc -l | tr -d ' ')
  [ "$uncommitted" -gt 0 ] && issues+=("$uncommitted uncommitted")

  # Current branch
  branch=$(git -C "$dir" branch --show-current 2>/dev/null)
  [ -n "$branch" ] && [ "$branch" != "main" ] && [ "$branch" != "master" ] && \
    issues+=("on $branch")

  # Unpushed commits
  unpushed=$(git -C "$dir" log --oneline @{u}..HEAD 2>/dev/null | wc -l | tr -d ' ')
  [ "$unpushed" -gt 0 ] && issues+=("$unpushed unpushed")

  # Unmerged branches
  main_branch="main"
  git -C "$dir" rev-parse --verify main >/dev/null 2>&1 || main_branch="master"
  unmerged=$(git -C "$dir" branch --no-merged "$main_branch" 2>/dev/null | grep -v '^\*' | wc -l | tr -d ' ')
  [ "$unmerged" -gt 0 ] && issues+=("$unmerged unmerged branches")

  # Report if any issues
  if [ ${#issues[@]} -gt 0 ]; then
    echo "$repo: ${issues[*]}"
  fi
done
```

## Output Format

```
repo-name: N uncommitted, on branch-name, M unpushed, K unmerged branches
```

## Triage Priority

1. **Uncommitted changes** - Highest risk of loss
2. **Unpushed commits** - Local only, at risk
3. **Unmerged branches** - Safe on remote but incomplete
4. **Non-main branches** - Context indicator, not necessarily risky

## Common Patterns

| Pattern | Likely Cause | Action |
|---------|--------------|--------|
| Many uncommitted in one repo | Active work | Checkpoint commit |
| On feature branch, 0 uncommitted | Paused work | Note context, may resume later |
| Unpushed commits | Forgot to push | Push or verify intentional |
| Many unmerged branches | Branch hygiene needed | Review and clean up |
