---
name: git-final-pass
disable-model-invocation: true
description: |
  Pre-PR quality checklist encoding recurring review feedback. Run before
  creating a PR to catch common mistakes. Covers: error handling, naming,
  resource cleanup, test coverage, documentation, commit hygiene, and
  language-specific gotchas. Use when code is "done" and about to be PR'd.
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# Final Pass

Run this checklist before creating a PR. Every item is a mistake that has actually happened.

## Process

1. Identify changed files: `git diff --name-only HEAD~1` or `git diff --name-only main`
2. Run each check category against changed files only
3. Produce a pass/fail table
4. Fix failures before proceeding to PR

## Checks

### Error Handling

| Check | What to look for |
|-------|-----------------|
| Unchecked errors | Go: `err` assigned but not checked. TS: `.catch()` missing on promises |
| Swallowed errors | `catch {}` or `if err != nil { return nil }` with no logging |
| Error message quality | Does the error say what went wrong AND what was being attempted? |
| Partial failure | If a multi-step operation fails mid-way, is state left consistent? |
| Context propagation | Go: `fmt.Errorf("doing X: %w", err)` not `return err` bare |

### Naming

| Check | What to look for |
|-------|-----------------|
| Abbreviation | No abbreviations except universally understood (id, url, http, db) |
| Boolean naming | Booleans should read as questions: `isReady`, `hasPermission`, `canRetry` |
| Consistency | Same concept uses same name everywhere. Don't mix `user`/`account`/`principal` |
| Exported names | Go: exported names should not stutter (`user.UserName` -> `user.Name`) |

### Resource Cleanup

| Check | What to look for |
|-------|-----------------|
| Open/close pairing | Every open (file, connection, channel) has a corresponding close |
| Defer placement | Go: `defer` immediately after successful open, not after error check |
| Context cancellation | Go: `context.WithCancel` always has `defer cancel()` |
| Goroutine lifetime | Every goroutine has a clear termination condition |

### Tests

| Check | What to look for |
|-------|-----------------|
| New code has tests | Every new function/method has at least one test |
| Tests have failure hypothesis | Each test name describes what breaks: `TestRejectsExpiredToken` |
| No weakened tests | Tests weren't modified to make them pass (weaker assertions, removed cases) |
| Edge cases | Empty input, nil/null, zero values, boundary conditions |
| Error paths tested | Not just happy path — test what happens when things fail |

### Documentation

| Check | What to look for |
|-------|-----------------|
| Public API documented | Exported functions/types have doc comments |
| Changed behavior noted | If behavior changed, docs updated to match |
| README current | If user-facing changes, README reflects them |
| No stale comments | Comments near changed code still accurate |

### Commit Hygiene

| Check | What to look for |
|-------|-----------------|
| No unrelated changes | Every file in the diff relates to the PR's purpose |
| No debug artifacts | No `console.log`, `fmt.Println`, `print()` left in |
| No commented-out code | Dead code should be deleted, not commented |
| No TODOs without tickets | New TODOs reference a tracking number or explain why they exist |
| No secrets | No API keys, tokens, passwords, or internal URLs |
| .gitignore current | Generated files (binaries, build output) are ignored |

### Language-Specific: Go

| Check | What to look for |
|-------|-----------------|
| Nil pointer | Interface values checked for nil before method calls |
| Slice nil vs empty | `nil` slice and `[]T{}` behave differently in JSON marshaling |
| Map initialization | Maps must be initialized before write (`make(map[K]V)`) |
| Race conditions | Shared state across goroutines protected by mutex or channel |
| Import organization | stdlib, external, internal — separated by blank lines |

### Language-Specific: TypeScript

| Check | What to look for |
|-------|-----------------|
| Any types | No `any` unless genuinely unavoidable (document why) |
| Null handling | Optional chaining (`?.`) or explicit null checks, not `!` assertions |
| Async error handling | Every `async` function has error handling or propagates correctly |
| Type narrowing | Union types narrowed before use, not cast with `as` |

## Common Mistakes

These are the mistakes that show up most often in review:

1. **Error returned but not wrapped** — bare `return err` loses context. Wrap with what was being attempted.
2. **Test added but only tests happy path** — if the function can fail, test the failure.
3. **Resource opened in a loop, closed after** — must close each iteration or collect for batch close.
4. **Boolean parameter** — `doThing(true, false)` is unreadable. Use named options or separate functions.
5. **Magic numbers** — `if len(items) > 100` — where does 100 come from? Name it.
6. **Goroutine leak** — launched goroutine has no way to stop when parent context is cancelled.
7. **Logging sensitive data** — user tokens, credentials, PII in log output.
8. **Changed behavior, unchanged tests** — modified a function but the old tests still pass because they're too loose.

## Output

Produce a table:

```
Category            Status  Notes
---                 ---     ---
Error handling      PASS
Naming              PASS
Resource cleanup    FAIL    db connection not closed in syncBatch
Tests               PASS
Documentation       PASS
Commit hygiene      FAIL    console.log on line 47
Go-specific         PASS
```

Fix all FAILs before proceeding.
