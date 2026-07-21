---
name: large-scale-refactor
description: >
  Guardrails, protocols, and operating constraints for large-scale, long-running,
  or parallelized AI coding tasks — migrations, codebase-wide refactors, framework
  upgrades, and any task touching 50+ files. Prevents scope creep, context drift,
  silent compounding errors, and emergent behavior outside the defined task boundary.
  Use when refactoring across files, migrating frameworks, upgrading dependencies,
  replacing or renaming patterns throughout a codebase, or any task touching 50+
  files.
license: MIT
author: opensite-ai
version: 1.0.0
tags:
  - refactor
  - migration
  - long-running
  - multi-agent
  - guardrails
  - agentic
activation_patterns:
  - "refactor * across"
  - "migrate * to"
  - "upgrade * from"
  - "replace all"
  - "update every"
  - "rename * throughout"
  - "convert all"
  - "remove all instances"
  - "batch * across the codebase"
  - "files_touched_estimate >= 50"
---

# large-scale-refactor

Activates when a task will touch 50+ files, run longer than one agent session, or be parallelized across instances. Once active, task scope is locked: no deviation, even for apparent related improvements.

## Spec gate

No execution without a written, human-approved spec. Halt after producing the spec and await explicit approval before writing any code. In parallel mode, the spec is approved before spawning any instances; the approved spec is the canonical context injected into every instance.

Spec contents:
- Task name, date, initiator
- One concrete paragraph of what the task does
- IN SCOPE: file types, operations, directories
- OUT OF SCOPE (do not touch): styling/theming/color tokens, business logic and data transformation behavior, component structure beyond what the task requires, dependencies (beyond `@types/*` for TS migrations), build/CI/deploy configs, and anything not explicitly IN SCOPE
- Decomposition into atomic, independently reviewable subtasks (each: N files, directory)
- Acceptance criteria: in-scope files at target state; tests pass or failures are pre-existing and documented; no net-new deps beyond spec; no out-of-scope files modified; one commit/PR per subtask
- Rollback plan (branch name, revert strategy)

## Scope enforcement

- One job: the spec. Bugs, improvements, missing tests, perf wins, style issues not in spec — log to `OBSERVATIONS.md` (file | observation | severity) and leave alone. Never act on observations.
- Substitution test before every change: "If I remove this change from the diff, does the task still fail?" If the task still succeeds without it, do not make the change.
- No emergent systems unless named in the spec: no new design/theming systems, utility libraries or helper modules, abstractions/base classes/shared components, folder reorganization, build steps/scripts/tooling, config files or env vars, test harnesses, or any file that did not exist before the task (exception: spec-defined outputs, e.g. `.ts` replacing `.js`). If a shared utility is genuinely needed, propose it in OBSERVATIONS.md and halt for approval.
- No dependency add/remove/upgrade unless spec-listed, or a `@types/*` package in a TS migration.
- 50-line rule: more than 50 lines of net-new logic for one change (excluding type annotations, renames, reformatting) means scope creep or a task needing architectural discussion. Stop, log, halt for review.

## Execution

- Pilot batch: process the first batch with only 10–20 files, even if the risk level allows more. Never skip — it surfaces spec edge cases and validates the transformation before scaling.
- One commit/PR per subtask, never combined. Message: `refactor(<subtask-id>): <description>` plus task name, file count, spec ref.
- File budget per session/instance:

  | Risk | Max files/session | Review cadence |
  |------|-------------------|----------------|
  | Low (type renames, import fixes) | 200 | End of session |
  | Medium (logic-adjacent refactors) | 50 | Every 25 files |
  | High (framework migrations, API changes) | 20 | Every 10 files |

  At budget: commit, push, stop. Human reviews before the next session.
- Parallel instances: each receives the approved spec as its first system message (overrides in-session reasoning); non-overlapping file sets assigned by directory or explicit list — never "all files matching X" without per-instance assignment; read-only toward shared abstractions (a needed one is flagged and human-created before instances consume it); no communication with or observation of other instances — context contamination between parallel agents is a primary source of emergent behavior. Monitor the first 10 files of any new parallel run for out-of-scope touches.
- Drift check at the review cadence (default: every 25 files): every changed file in IN SCOPE? new files beyond spec? dependency changes? changes failing the substitution test? new abstractions? Any yes: HALT, surface to human. All no: attach the check log to the commit message.

## Human checkpoints

Hard stops — do not continue until a human explicitly clears them:
spec gate; drift-check failure; out-of-scope file touched or discovered; new dependency needed; new shared abstraction needed; build/tests fail in a way the spec didn't anticipate; file budget reached; ambiguity about whether a file is in scope (ask, never assume).

Checkpoint message: trigger, context, options (always include abort-and-preserve), recommendation. No changes while awaiting response.

## Verification per subtask

Produce `CHANGE_MANIFEST.md`: counts of files modified/created/deleted (created should be 0 for pure refactors), per-file change table, scope-compliance checklist, test results before/after distinguishing new vs pre-existing failures.

Run and record before marking a subtask complete:

```bash
git diff HEAD package.json package-lock.json yarn.lock Cargo.toml Gemfile*   # dep check
git diff HEAD --name-status | grep "^A"                                      # new-file audit
git diff HEAD --name-only | grep -v -f .refactor-scope-allowlist             # scope check
# then: test suite, lint
```

`.refactor-scope-allowlist` is generated from the spec's IN SCOPE list; any file surviving the scope check triggers a checkpoint.

## Cross-session context

- `.refactor-session.md`: written at end of every session, read at start of the next, committed with the session's changes. Contents: completed/in-progress/remaining subtasks, files remaining, decisions and edge cases to carry forward, active blockers, spec ref. This is what lets a fresh context (new session, different model) resume without drift.
- Context flush after each batch (performance degrades as context fills with diffs): commit and push, write the handoff file, discard the batch's diffs and intermediate reasoning, reload only the spec + latest handoff + next batch file list. Flush immediately if: referencing a decision more than 2 batches old without consulting the handoff file; uncertain about the original task without re-reading the spec; making changes that "feel right" by pattern matching rather than explicit spec compliance.
