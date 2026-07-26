---
name: git-create-pr
disable-model-invocation: true
description: |
  Safe git-to-PR workflow. Classifies changes, stages by explicit filename,
  runs final-pass checks, commits with approval, pushes, creates PR via gh.
  Encodes all git safety rules. Use when ready to ship code as a PR.
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
  - AskUserQuestion
  - Agent
  - Skill
---

# Create PR

Complete workflow from dirty working tree to open pull request.

## Prerequisites

- On a feature branch (not main/master)
- Branch follows naming convention: `rch/<type>/<topic>`
- Changes are ready for review

## Process

### Phase 1: Inventory

1. Run `git status` to see all changes
2. Run `git diff --stat` to see scope
3. Classify each changed file:
   - **Core**: directly related to the PR's purpose
   - **Supporting**: tests, docs, configs for core changes
   - **Incidental**: formatting, typo fixes discovered along the way
   - **Unrelated**: changes that don't belong in this PR (stash or reset these)

Present the classification to the user. Ask if any files should be excluded.

### Phase 2: Pre-flight

Run `/git-final-pass` on the changes. Fix any failures before proceeding.

If the project has:
- A linter: run it (`make lint`, `golangci-lint run`, `npx eslint`, etc.)
- Tests: run them (`make test`, `go test ./...`, `npm test`, etc.)
- Type checking: run it (`make check`, `npx tsc --noEmit`, etc.)

All must pass before proceeding. Do not skip failing checks.

### Phase 3: Stage

Stage files by explicit name. **Never use `git add -A` or `git add .`**

```bash
git add path/to/file1.go path/to/file2.go path/to/file2_test.go
```

Verify staged files match the classification from Phase 1:
```bash
git diff --cached --name-only
```

### Phase 4: Commit

Draft a commit message:
- Short subject line (imperative mood, <72 chars)
- Blank line
- Body explaining what and why (not how)
- No Co-Authored-By or Signed-off-by trailers

Show the message to the user. **Ask "ready to commit?" and wait for approval.**

### Phase 5: Push

Check if branch tracks a remote:
```bash
git rev-parse --abbrev-ref @{upstream} 2>/dev/null
```

If not, push with `-u`:
```bash
git push -u origin HEAD
```

**Ask "ready to push?" and wait for approval.** Never push without asking.

### Phase 6: Create PR

Use `gh pr create`:
- Title: short, under 70 chars, describes the change
- Body: what changed, why, how to test
- Use casual-slack-tone for own repos, dry-witted-engineering for others

```bash
gh pr create --title "title" --body "$(cat <<'EOF'
## What

Brief description of changes.

## Why

Motivation and context.

## Test plan

How to verify this works.
EOF
)"
```

Return the PR URL to the user.

## Safety Rules

1. Never `git add -A` or `git add .` — stage files explicitly
2. Never commit without user approval
3. Never push without user approval
4. Never force push (`--force`, `--force-with-lease`) without explicit instruction
5. Never skip hooks (`--no-verify`)
6. Never commit secrets, credentials, or internal URLs
7. Never include unrelated changes in the PR
8. If lint/test/check fails, fix it — don't skip it

## Common Mistakes

1. **Staging generated files** — check .gitignore covers build output, compiled binaries
2. **Committing debug output** — search for `fmt.Println`, `console.log`, `print()` in staged diff
3. **PR title too vague** — "updates" or "fixes" says nothing. Name the thing that changed.
4. **Missing test plan** — reviewer can't verify without knowing how to test
5. **Huge PR** — if staged diff is >500 lines, consider splitting into smaller PRs
6. **Wrong branch base** — verify PR targets the right base branch (usually main)
