---
name: git-create-pr
disable-model-invocation: true
description: |
  Safe git-to-PR workflow. Classifies changes, stages by explicit filename,
  runs final-pass checks, commits verified work without a second ask, pushes
  under the Claude.md draft-PR and clean-rebase exceptions, creates PR via gh.
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


## Common Mistakes

1. **Staging generated files** — check .gitignore covers build output, compiled binaries
2. **Committing debug output** — search for `fmt.Println`, `console.log`, `print()` in staged diff
3. **PR title too vague** — "updates" or "fixes" says nothing. Name the thing that changed.
4. **Missing test plan** — reviewer can't verify without knowing how to test
5. **Huge PR** — if staged diff is >500 lines, consider splitting into smaller PRs
6. **Wrong branch base** — verify PR targets the right base branch (usually main)

## Prerequisites

- On a feature branch (not main/master)
- Branch follows naming convention: `<username>/<type>/<topic>`
- Changes are ready for review

## Process

### Phase 1: Inventory

1. Run `git status` to see all changes
2. Run `git diff --stat` to see scope
3. Classify each changed file:
   - **Core**: directly related to the PR's purpose
   - **Supporting**: tests, docs, configs for core changes
   - **Incidental**: formatting, typo fixes discovered along the way
   - **Unrelated**: changes that don't belong in this PR (leave them in the tree; do not stash when other agents share the tree)

If the brief already named the include list, skip the exclude ask. Stop after commit when the brief says do not push. Otherwise present the classification.

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

Show the commit message. Commit verified work without waiting.

### Phase 5: Push

Check if branch tracks a remote:
```bash
git rev-parse --abbrev-ref @{upstream} 2>/dev/null
```

If not, push with `-u`:
```bash
git push -u origin HEAD
```

Push without asking only for a draft PR of verified work, a clean rebase with --force-with-lease, or rebase-plus-docs. Merge without asking a remaining-diff-only tests/docs/lint/lsp PR with CI green on the merge SHA. Ask before a non-draft push, marking ready, or merge of any other PR.

### Phase 6: Create PR

Use `gh pr create`:
- Title: short, under 70 chars, describes the change
- Body: what changed, why, how to test
- Use casual-slack-tone for own repos, dry-engineering for others (no wit)

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
2. Commit verified work freely after the user already asked to ship.
3. Never push a non-draft publish without asking, except the draft-PR and clean-rebase exceptions in Claude.md.
4. Never `--force`. `--force-with-lease` only for a clean rebase of already-authored commits, per Claude.md.
5. Never skip hooks (`--no-verify`)
6. Never commit secrets, credentials, or internal URLs
7. Never include unrelated changes in the PR
8. If lint/test/check fails, fix it — don't skip it

## Before finishing

- [ ] Common Mistakes checked against this run?
- [ ] Required outputs exist (PR/branch/status)?
- [ ] No trailers in published text?
