---
name: review-code
description: Run a multi-agent code review on recent changes. Spawns a team of specialized reviewers (bugs, security, perf, tests, usability, etc.), collects findings, triages into fix-now vs defer, and optionally applies fixes. Use when asked to "review code", "review changes", "review this PR", or "get a second opinion".
disable-model-invocation: true
argument-hint: [--apply] [--defer-file <path>]
---

# Multi-Agent Code Review

Spawn a team of specialized reviewer agents to review recent code changes. Each reviewer examines the code from a distinct perspective, reports findings with severity and file:line references, and the lead synthesizes a unified report with triage recommendations.

Original skill by Bjorn Tipling. Enhanced with project-local reviewer integration.

## Arguments

Usage: `/review-code [--apply] [--defer-file <path>]`

- `--apply` — After presenting findings, automatically apply the "fix now" batch without asking.
- `--defer-file <path>` — Append deferred items to this file (e.g., a plan doc or TODO). Default: do not write deferred items to a file.

## Step 1: Determine Scope

Identify what code to review:

1. Check `git status` and `git diff` for uncommitted changes.
2. Check `git log --oneline main..HEAD` for commits on the current branch.
3. If on a branch with commits ahead of main, review the full branch diff: `git diff origin/main...HEAD`.
4. If only uncommitted changes, review those.
5. Collect the list of changed files (exclude `vendor/`, generated files, `go.sum`).

Store as `CHANGED_FILES` and `FULL_DIFF`.

## Step 2: Select Reviewers

Based on the changes, select which reviewers to spawn. The following are the **suggested defaults** — add, remove, or adjust based on what the code actually touches. Not every review needs every reviewer. Use judgment.

### Consult project-local reviewer skills

Before finalizing the roster, check the current project for domain-specific review skills:

1. Look for `skills/CATALOG.md` in the project root.
2. Look for `skills/*-review/SKILL.md` files (convention: skills whose name ends in `-review` are reviewer personas).
3. For each project-local review skill, consider whether the changeset touches its domain. Spawn it as an additional reviewer if it applies.

**Example:** In the occult repo, `skills/occult-source-review/SKILL.md` applies when any `.occult` file changed. `skills/occult-engine-review/SKILL.md` applies when Go engine packages (`ir/`, `state/`, `solve/`) changed. Spawn them alongside the general-purpose defaults below.

Project-local skills encode domain conventions (naming rules, idioms, invariants) that generic reviewers miss. Always prefer them over inventing domain criteria from scratch.

### Suggested Reviewer Roster

| Reviewer | Focus | When to include |
|----------|-------|-----------------|
| **bugs-reviewer** | Logic bugs, edge cases, error handling gaps, off-by-one errors, nil/zero-value hazards | Always — this is the baseline reviewer |
| **security-reviewer** | Path traversal, injection, credential leakage, DoS vectors, OWASP concerns | When code handles external input, file I/O, exec, network, or auth |
| **perf-reviewer** | Memory usage, goroutine leaks, unnecessary allocations, timeout calibration, resource limits | When code does I/O, spawns processes, handles large data, or has concurrency |
| **test-reviewer** | Test coverage gaps, missing edge case tests, test quality, flaky test risks, test isolation | When the changeset includes or should include tests |
| **usability-reviewer** | API ergonomics, error message clarity, documentation quality, discoverability | When code exposes a public API, CLI, MCP tools, or config schema |

### Additional Reviewers to Consider

These are not default — add them when the context calls for it:

| Reviewer | Focus | When to consider |
|----------|-------|------------------|
| **integration-reviewer** | SDK/framework usage correctness, comparing against reference implementations | When using a specific SDK or framework with known patterns |
| **arch-reviewer** | Architectural fit, separation of concerns, abstraction boundaries | For large structural changes or new subsystems |
| **compat-reviewer** | Breaking changes, backwards compatibility, migration paths | When modifying public APIs or data formats |

### Deciding Which Reviewers to Use

- **Small bugfix** (1-3 files, one concern): bugs-reviewer only, maybe test-reviewer.
- **New feature** (new files, new API surface): bugs + security + usability + test.
- **Performance work**: bugs + perf + test.
- **Full PR review**: all five defaults.
- **Never spawn more than 6 reviewers** — diminishing returns, and the synthesis step gets unwieldy. When project-local skills push the count over 6, prefer the project-local ones (they're more targeted) and drop generic defaults that overlap (e.g., drop `bugs-reviewer` if the project has a domain-specific bug-pattern reviewer).

## Step 3: Create Review Team

1. Create a team: `TeamCreate` with name `code-review` and description based on the changes.
2. Create one task per reviewer in the team's task list.
3. Spawn all selected reviewers **in parallel** using the Agent tool.

### Reviewer Spawn Template

Each reviewer agent should receive this context in its prompt:

```
You are a {REVIEWER_ROLE} reviewing code changes in {PROJECT_PATH}.

CHANGED FILES: {CHANGED_FILES}

YOUR TASK:
1. Read each changed file in full (not just the diff — you need surrounding context)
2. Review against your specific criteria (described below)
3. Report findings as a numbered list with:
   - Severity: Critical / High / Medium / Low / Nit
   - File:line reference
   - Description of the issue
   - Suggested fix or mitigation
4. If an area is clean, say so explicitly — "no issues found in X" is valuable signal
5. Claim your task from the task list when you start, mark it completed when done
6. Send your findings to team-lead via SendMessage when complete

YOUR CRITERIA:
{REVIEWER_SPECIFIC_CRITERIA}

IMPORTANT: Only report findings with confidence >= 80%. Verify by reading actual code before reporting.
```

For reviewers derived from **project-local skills**, set `REVIEWER_SPECIFIC_CRITERIA` to: "Read `{project-skill-path}/SKILL.md` and apply its checklist verbatim. Flag any violations." This delegates criteria authorship to the project.

### Reviewer Criteria

**bugs-reviewer:**
- Logic errors: wrong conditionals, off-by-one, nil dereferences
- Error handling: unchecked errors, swallowed errors, wrong error wrapping
- Edge cases: empty inputs, zero values, boundary conditions
- Concurrency: race conditions, deadlocks, goroutine leaks
- Contract violations: io.Writer, io.Closer, interface compliance
- Test coverage: are important code paths tested?

**security-reviewer:**
- Input validation: path traversal, injection (SQL, command, template)
- File I/O: symlink following, TOCTOU races, size limits
- Exec: command injection, argument injection, environment leakage
- Credentials: secrets in logs/errors, hardcoded credentials
- Resource exhaustion: unbounded allocations, missing timeouts, DoS vectors
- Output: information leakage in error messages

**perf-reviewer:**
- Memory: unnecessary copies, unbounded buffers, large allocations on hot paths
- I/O: missing timeouts, unbuffered I/O, connection leaks
- Concurrency: goroutine leaks, contention, unnecessary synchronization
- Allocations: regex compilation in loops, repeated marshaling, string concatenation
- Limits: are size/time/count limits well-calibrated?
- Context propagation: is context.Context threaded correctly?

**test-reviewer:**
- Coverage: are all public functions tested? All error paths?
- Edge cases: empty inputs, nil values, boundary values, large inputs
- Test quality: are assertions specific enough? Do tests verify behavior, not implementation?
- Isolation: do tests depend on external services, file system state, or test ordering?
- Determinism: any flaky test risks (timing, random data, network)?
- Missing tests: what test cases should exist but don't?

**usability-reviewer:**
- API design: are function signatures intuitive? Are parameter names clear?
- Error messages: are they actionable? Do they tell the user what to fix?
- Documentation: are public types and functions documented? Are docs accurate?
- Discoverability: can users find features via help, schemas, or autocomplete?
- Consistency: does the API match conventions in the rest of the codebase?
- Edge case UX: what happens with unusual but valid inputs?

## Step 4: Collect and Synthesize Findings

Wait for all reviewers to complete (check task list, collect SendMessage responses).

If a reviewer goes idle without sending findings, prompt them once. If still no response after a second prompt, proceed without their input and note the gap.

### Synthesize

1. **Deduplicate**: Multiple reviewers may flag the same issue (e.g., bugs + security both flag missing input validation). Keep the most detailed finding, note which reviewers flagged it.
2. **Cross-validate**: If findings seem contradictory, read the code yourself to resolve.
3. **Sort by severity**: Critical > High > Medium > Low > Nit.

## Step 5: Triage

Classify each finding into one of two buckets:

### Fix Now
Criteria (any of):
- Critical or High severity
- Quick fix (< 5 minutes, < 20 lines changed)
- Blocks correctness or basic usability
- The fix is unambiguous (no design decision needed)

### Defer
Criteria (any of):
- Needs a design decision or discussion
- Large scope (new abstraction, significant refactor)
- Low real-world risk (theoretical concern, defense-in-depth)
- Requires changes outside the current changeset

## Step 6: Present Report

Present the synthesized report to the user in this format:

```
## Code Review Report

**Scope**: {branch or description}
**Reviewers**: {list of reviewers spawned}
**Files reviewed**: {count}

### Summary

| Severity | Count |
|----------|-------|
| Critical | X |
| High     | Y |
| Medium   | Z |
| Low      | W |
| Nit      | N |

### Fix Now (X items)

{numbered list with severity, file:line, description, suggested fix}

### Defer (Y items)

{numbered list with severity, description, rationale for deferring}

### Clean Areas

{what the reviewers explicitly confirmed as clean}
```

## Step 7: Apply Fixes (if requested or approved)

If `--apply` was passed, or the user approves:

1. Apply each "Fix Now" item.
2. Run build and lint to verify.
3. Run tests to verify no regressions.
4. Present a summary of what was changed.

If `--defer-file` was specified, append the deferred items to that file with enough context to act on later (severity, description, rationale, suggested approach).

## Step 8: Clean Up

Shut down all reviewer teammates and clean up the team.
