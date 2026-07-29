---
name: find-delegation-pebbles
description: Sweep open issue trackers, TODOs, active branches, and source evidence to find narrowly scoped, independent, verifiable tasks suitable for remote agents or parallel delegation. Use when asked to find pebbles, small delegatable work, fire-and-forget candidates, backlog slices, ready items, or pool/task combinations.
---

# Find Delegation Pebbles

Produce an evidence-grounded delegation queue. A pebble is not merely small:
it is self-contained, collision-resistant, and cheaply verifiable.

Do not dispatch work unless the user asks for execution. Discovery and pool
packing are useful independently.

## Workflow

1. Read repository instructions and the applicable remote-execution skill.
2. Inventory the complete candidate population:
   - open and ready tracker items;
   - incomplete-work markers and canonical TODO documents;
   - in-progress items, open pull requests, branches, and worktrees;
   - current source and tests for each shortlisted item.
3. Remove stale, completed, duplicated, blocked, and already-active work.
4. Apply the eligibility gates and score the survivors.
5. Combine overlapping micro-items into one ownership unit.
6. Pack independent units into homogeneous pools.
7. Return ready, decompose-first, and reject/defer groups with evidence.

## Hard Eligibility Gates

A dispatch-ready pebble must satisfy every gate:

- **One outcome:** one observable deliverable, even if it closes several
  duplicate tracker entries.
- **Bounded ownership:** normally one package or at most three tightly related
  files. Broader mechanical sweeps must have a deterministic file partition.
- **No unresolved decision:** no pending architecture, product, security,
  language, syntax, public-API, or compatibility choice.
- **No hidden dependency:** no required unmerged branch, active parent change,
  unavailable service, credential, or human approval.
- **Independent merge:** no overlap with active work or another task in the
  same pool. Combine overlapping items or sequence them from an explicit base.
- **Executable acceptance:** expected behavior and failure behavior are stated
  precisely enough for a remote agent to finish without asking questions.
- **Bounded proof:** focused tests, lint, build, or a deterministic artifact can
  prove completion inside the remote environment.
- **Safe failure:** a wrong implementation is reviewable and reversible; it
  cannot silently alter production, credentials, published language semantics,
  or destructive state.

Treat parser, language, stdlib-contract, and author-visible semantic changes as
not dispatch-ready until explicit approval is recorded.

Treat P0/P1 soundness work as supervised even when implementation is narrow:
delegation may produce a candidate patch, but the main session owns semantic
review and final verification.

## Score Survivors

Score each dimension from 0 to 2:

- scope boundedness;
- requirement clarity;
- file/branch isolation;
- focused verification quality;
- environment self-sufficiency;
- merge/review simplicity.

Only call an item ready when it passes every hard gate and scores at least
10/12. Record the reason for every deduction. Confidence follows evidence:
source and test inspection outrank tracker wording.

## Detect False Pebbles

Reject or reclassify these common shapes:

- a one-line fix whose correct behavior is unresolved;
- a catch-all issue containing unrelated findings;
- two issues describing the same defect;
- a task already implemented on the current main branch;
- a test task whose only gate is the full repository suite;
- a mechanical sweep over files another branch is actively changing;
- local-worktree, stash, credential, or operator-state cleanup;
- a research or design verb presented as implementation;
- a language-facing change hidden inside a lint, fixture, or documentation task.

Very small edits may cost less locally than remote setup. Group related
micro-items by ownership surface, or classify them as local pebbles.

## Pack Pools and Tasks

Use one pool only when tasks share the same repository, image, toolchain, and
gate family. Parallel tasks must own disjoint files or packages.

Prefer:

- one environment/pool with sibling tasks for a homogeneous repository;
- one cohesive task for several items touching the same central file;
- separate tasks for disjoint packages;
- a separate broad-verification pool after focused task gates pass.

Use the repository-approved remote execution workflow. Create sibling tasks in
one verified environment when the provider supports them. Specify environment,
working directory, base revision, title, prompt, model, and idempotency key.
Do not route through deprecated pool wrappers.

Never substitute a bare full-suite prompt for focused completion gates. Broad
suites need an explicit, separately owned verification task after focused
task gates pass.

## Dispatch Manifest

When preparing tracker items for later dispatch, add:

```text
## Remote Dispatch
- Pool: repository / image / flavor
- Task: one concrete outcome
- Owns: exact files or package
- Must not touch: collision and scope boundaries
- Base: required ref or dependency
- Gates: focused commands and expected evidence
- Stop conditions: ambiguity, unsupported behavior, wider changes
- Language/API: none, approved, or blocked
```

Do not claim or dispatch an item during a discovery-only request.

## Output

Report:

1. population size and sources searched;
2. ready-now pebbles, ranked by score;
3. proposed pool/task packing;
4. combine-or-decompose items;
5. stale, duplicate, active, or unsafe exclusions;
6. current infrastructure blockers;
7. the smallest next executable batch.

For every ready item include its tracker ID, outcome, ownership surface,
focused gate, language/API risk, dependencies, score, and confidence.
