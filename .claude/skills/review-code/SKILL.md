---
name: review-code
description: Run a multi-agent code review on recent changes. Spawns a team of specialized reviewers (bugs, security, perf, tests, usability, etc.), collects findings, triages into fix-now vs defer, and optionally applies fixes. Use when asked to "review code", "review changes", "review this PR", or "get a second opinion".
disable-model-invocation: true
argument-hint: [--apply] [--defer-file <path>]
---

# Multi-Agent Code Review

Spawn parallel reviewer agents over recent changes; synthesize one triaged report.

Original skill by Bjorn Tipling.

## Arguments

- `--apply` — apply the "fix now" batch after presenting findings, without asking.
- `--defer-file <path>` — append deferred items to this file. Default: deferred items are not written to any file.

## Scope

If the branch is ahead of main, review the full branch diff (`git diff origin/main...HEAD`); otherwise review uncommitted changes. Exclude `vendor/`, generated files, `go.sum`.

## Select Reviewers

First check the project for domain-specific reviewer skills: `skills/CATALOG.md` and `skills/*-review/SKILL.md` (convention: names ending in `-review` are reviewer personas). Spawn any whose domain the changeset touches. Prefer these over generic defaults — they encode conventions generic reviewers miss. Their criteria prompt is: "Read `{skill-path}/SKILL.md` and apply its checklist verbatim. Flag any violations."

Generic roster — pick per what the code touches:

| Reviewer | Focus | Include when |
|----------|-------|--------------|
| bugs-reviewer | Logic bugs, edge cases, error handling, concurrency | Always — baseline |
| security-reviewer | Injection, path traversal, credentials, DoS | External input, file I/O, exec, network, auth |
| perf-reviewer | Allocations, leaks, timeouts, limit calibration | I/O, subprocesses, large data, concurrency |
| test-reviewer | Coverage gaps, test quality, isolation, flakiness | Changeset includes or should include tests |
| usability-reviewer | API ergonomics, error messages, docs, discoverability | Public API, CLI, MCP tools, config schema |
| integration-reviewer | SDK/framework usage vs reference patterns | Non-default: specific SDK with known patterns |
| arch-reviewer | Architectural fit, abstraction boundaries | Non-default: large structural changes |
| compat-reviewer | Breaking changes, migration paths | Non-default: public API or data format changes |

Never spawn more than 6 reviewers. If project-local skills push past 6, drop the generic ones that overlap (e.g. drop bugs-reviewer if the project has a domain bug-pattern reviewer).

## Spawn Team

Create a team via `TeamCreate` (name `code-review`), one task per reviewer, then spawn all reviewers in parallel with the Agent tool. Each reviewer prompt must include:

- The changed files, and the instruction to read each in full — not just the diff.
- Its review criteria (from the Focus column or the project-local skill).
- Finding format: numbered list, each with severity (Critical/High/Medium/Low/Nit), file:line, description, suggested fix.
- Report clean areas explicitly — "no issues found in X" is signal.
- Claim its task when starting, complete it when done; send findings to team-lead via SendMessage.
- Only report findings with confidence >= 80%, verified by reading the actual code before reporting.

## Synthesize

If a reviewer goes idle without sending findings, prompt it once; if still silent after a second prompt, proceed without it and note the gap.

Deduplicate (keep the most detailed finding, note which reviewers flagged it), cross-validate contradictions by reading the code yourself, sort by severity.

## Triage

- **Fix Now** (any of): Critical or High; quick fix (< 5 minutes, < 20 lines); blocks correctness or basic usability; fix is unambiguous.
- **Defer** (any of): needs a design decision; large scope; low real-world risk; requires changes outside the changeset.

## Report

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

## Apply

If `--apply` or the user approves: apply each Fix Now item, verify with build/lint/tests, summarize what changed.

If `--defer-file`: append deferred items with enough context to act on later (severity, description, rationale, suggested approach).

## Clean Up

Shut down all reviewer teammates and clean up the team.
