---
name: git-pr
disable-model-invocation: true
description: |
  Safe git-to-PR workflow. Stages changes, runs quality checks, commits with
  approval, pushes, creates PR. Use when ready to ship code.
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
  - AskUserQuestion
  - Agent
  - Skill
---

# Git PR Workflow

Complete workflow from dirty tree to merged PR.

## Phases

### 1. Classify changes
Run `git diff --stat` and `git status`. Group by: feature code, tests, config, docs.

### 2. Quality check
Before staging, verify:
- [ ] No debug prints, console.logs, or TODO-as-code left behind
- [ ] Error handling: no swallowed errors, no bare `catch {}`
- [ ] Resource cleanup: anything opened is closed (files, connections, channels)
- [ ] Naming: functions describe behavior, variables describe content
- [ ] Tests exist for new behavior, regression tests for fixes
- [ ] No hardcoded secrets, paths, or credentials

### 3. Stage explicitly
`git add` specific files by name. Never `git add -A` or `git add .` without reviewing.

### 4. Commit
Show the diff summary and proposed message. Wait for approval.
Message format: imperative, under 72 chars, describes the why.

### 5. Push
`git push -u origin HEAD`. Wait for approval before pushing.

### 6. Create PR
Use `gh pr create`. Title under 70 chars. Body has Summary (bullets) and Test Plan.

## Common Mistakes

1. Staging generated files (binaries, .env, node_modules)
2. Committing before tests pass
3. Vague commit messages ("fix stuff", "updates")
4. Pushing to main instead of a feature branch
5. Including unrelated changes in the same PR
6. Forgetting to set the upstream branch
