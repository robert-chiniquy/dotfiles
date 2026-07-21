---
name: post-change-verification
description: Mandatory verification protocol after code changes for Go projects. Use after any code modification to ensure quality.
---

# Post-Change Verification Protocol

After creating, modifying, generating, or refactoring Go code (not after read-only analysis, audits, or docs-only work), run IN ORDER — prefer make targets over direct commands:

1. Format: `make fmt` (else `go fmt ./...`)
2. Lint: `make lint` (else `golangci-lint run`)
3. Build: `make build` (else `go build ./...`)
4. Test: `make test` (else `go test ./path/to/modified/...`, or `go test ./...`)

Target: ZERO errors and ZERO warnings across all steps.

## Issue Handling

1. Caused by your changes: fix before completing the task.
2. Pre-existing, in a file you modified, < 10 lines to fix: fix as part of current work.
3. Pre-existing, in a file you modified, >= 10 lines: document, report, recommend a separate cleanup task.
4. Pre-existing, in an unmodified file: document and report only; do not fix.

Pre-existing issues never block task completion; issues caused by your changes always do.

## Missing or Failing Commands

Do not block on tooling gaps — mark the step and continue:

- Command not found: `SKIPPED (command not found)`
- Runs > 5 min: `SKIPPED (timeout)`
- Execution error: `SKIPPED (execution error: [reason])`

## Report Format

Include this in task completion output:

```
=== POST-CHANGE VERIFICATION ===

Format:     [PASSED | FAILED | SKIPPED (reason)]
Lint:       [PASSED | FAILED] ([X] errors, [Y] warnings)
Build:      [PASSED | FAILED] ([X] errors)
Tests:      [PASSED | FAILED] ([X]/[Y] passed)

Pre-existing issues: [NONE | list with file:line]

=== [TASK COMPLETE | VERIFICATION FAILED] ===
```

Unfixed issues caused by the change end with `=== VERIFICATION FAILED - FIX ISSUES BEFORE COMPLETING ===`.

## Makefile

Go projects MUST have Makefile targets `fmt`, `lint`, `build`, `test` (plus combined `verify: fmt lint build test`). If targets are missing, fall back to direct commands and note it in the report.
