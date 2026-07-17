---
name: scramble
description: >-
  Tactically assemble a short-term plan of many parallel options, then pursue
  as much local action as possible in the moment — rebases, protogen, PR pushes,
  tracker state updates, sqfan fleet dispatch. Not a survey (orient/sitrep) and
  not a single-threaded task: a multi-option sprint where the agent fans out
  everything it can do locally right now. Triggers on "scramble", "max delta-v",
  "30 minute sprint", "parallel push", "many options at once", "as much local
  action as possible", "unblock and ship", "velocity push".
---

# scramble

## Overview

A scramble **assembles a short-term plan of many options**, then **pursues as many of them in parallel as local action allows** — all within a tight time box (default **30 minutes**).

It is not one task executed sequentially. It is:

1. **Survey the option space** — what could move in the next N minutes?
2. **Assemble a tactical plan** — pick the largest independent set of options worth doing *now*
3. **Fan out local execution** — run as many of those options simultaneously as the machine and repos allow
4. **Score delta-v** — what actually changed state?

Relationship to sibling skills:

| Skill | Mode |
|---|---|
| **sitrep** | Read one goal → A/B/C call |
| **orient** | Sweep everything → gestalt |
| **scramble** | Many options → parallel local action |

Bias toward **doing** over **describing**. No long reports, tracker comments, or planning docs unless the user asks.

## When to use

- User wants **maximum local action in the moment**
- User names a time box ("next 30 minutes", "before standup")
- Several in-flight threads could advance independently (PRs, branches, tracker states, fleet dispatches)
- Remote infra (squire, review queues) is slow — do everything locally first

## When NOT to use

- Read-only status → **tactical-sitrep** or **orient**
- Deep review of one PR → **pr-deep-review**
- User needs architectural choice before any code moves → plan first
- User explicitly wants tracker comments or docs → not scramble default

## Hard rules

1. **Options first, then parallel execution** — never jump to a single thread without scanning what else can move.
2. **Local action wins** — protogen, rebases, tests, pushes, state updates, batch file writes: do on host. Don't wait on squire for mechanical work.
3. **Execute yourself** — no runbooks for the user.
4. **Linear: state only** — status transitions, no comments (unless overridden).
5. **Always link PRs** when mentioning them.
6. **Independence is the fan-out limit** — parallelize options that don't share a dirty working tree or the same branch.

## Workflow

### Phase 1 — Pin the window (30 seconds)

State:

- Time budget (default 30m)
- Scope (milestone, project, or user-named goal)
- Metric: **count of state transitions** (merged, pushed, opened, dispatched, tracker state changed)

### Phase 2 — Build the option board (2–3 minutes)

Inventory what *could* move. Pull tracker items in scope + real-world signals (PRs, branches, CI, blockers). For each candidate, write one line on the **option board**:

```markdown
| Option | Type | Local? | Depends on | Est. |
|--------|------|--------|------------|------|
| Rebase c1 #20394 + re-review | close-in-review | yes | none | 10m |
| protogen cross-device branch | unblock-in-progress | yes | none | 15m |
| IGA-2446 → sqfan dispatch | fleet-dispatch | yes (brief) | 2263 branch exists | 5m |
| IGA-2218 → Todo | tracker-hygiene | yes | none | 30s |
| Merge c1 #20394 | close-in-review | no (human review) | Phoebe | — |
```

**Option types** (templates — add others as needed):

| Type | What it is |
|---|---|
| **close-in-review** | All or part of a chain ready to land; rebase blocker, re-request review, or mark Done if merged |
| **unblock-in-progress** | Branches exist; needs protogen, rebase, conflict fix, draft PR |
| **fleet-dispatch** | Todo item well-scoped for sqfan; write batch + fire |
| **tracker-hygiene** | Wrong state (In Progress with no work → Todo; stack visible → In Review) |
| **open-PR** | Branch pushed, no PR yet |
| **local-test** | Run targeted tests to unblock a push |
| **remote-blocked** | Needs human review, merge approval, or decision — note but don't pretend |

Mark **Local?** honestly. Remote-blocked options go on the board but not in the execution set.

### Phase 3 — Assemble the scramble plan (1 minute)

From the option board, select the **execution set**:

1. **Include** every option that is local, independent, and moves state.
2. **Prioritize** close-in-review and unblock-in-progress (highest delta-v per minute).
3. **Include** 1–3 fleet-dispatch options if Todo items qualify and won't steal focus from local pushes.
4. **Exclude** remote-blocked items from execution — list them under "waiting on others."
5. **Cap** total execution threads to what can run in parallel (typically 3–6 independent repo/tool operations).

Present the plan briefly — a numbered list the agent is about to run, not a document for the user to approve:

```markdown
## Scramble plan (30m)
1. [parallel] Rebase + push c1 #20394; request re-review
2. [parallel] protogen + push cross-device c1 branch; open draft PR
3. [parallel] Linear: IGA-2218/2245 → Todo; 2263/2264 → In Review
4. [parallel] Dispatch sqfan IGA-2446 batch
5. [if time] SDK branch rebase + push
```

Then **start all independent items immediately** — do not finish one before starting the next.

### Phase 4 — Parallel local execution (bulk of the window)

Fan out across independent options. Common local actions:

**Close in-review (per chain):**

- Map proto → server → SDK → CLI PRs
- If all merged → tracker **Done**
- If blocker PR open → rebase on main, fix conflicts, `make protogen`, targeted tests, push, re-request reviewers
- Do not mark Done while any required layer is still open

**Unblock in-progress:**

- Fetch + merge/rebase every repo in the chain
- Local protogen / `cargo test` / `go test` — fix only what breaks
- Push; open draft PRs for missing layers
- Tracker → **In Review** when stack is PR-visible

**Protogen (c1) — always local first:**

```bash
cd <c1-repo> && git checkout <branch>
git merge origin/main
# fix .proto manually; drop duplicate rpc stubs
make protogen
go test ./pkg/api/latchkey/ -run <Relevant> -count=1
git push origin <branch>
```

**Fleet-dispatch (Todo → sqfan):**

| Signal | Dispatch? |
|---|---|
| Well-scoped, ≤3 repos, clear deliverable | Yes |
| Needs running backend | Yes + `c1-dev-stack` |
| Decision / ADR / "decide whether" | No |
| Large platform / hardware greenfield | No |
| Blocked on in-flight stack merge | No |
| Adjacent parity work on existing branch family | Yes |

Write `sqfan-batches/<slug>/batch.yaml` + prompts; dispatch fire-and-forget via sqfan MCP. Cap at 2–3 per scramble.

**Tracker hygiene (throughout):**

| Condition | State |
|---|---|
| Stack PR-visible, CI running | In Review |
| All chain PRs merged | Done |
| No work, wrongly In Progress | Todo |
| Decision ticket | Todo |

### Phase 5 — Scorecard (last 2 minutes)

```markdown
## Scramble scorecard — <date> (<N>m)

**Plan:** <N> options assembled, <M> executed in parallel
**Delta-v:** <one sentence>

### Executed
- …

### Landed (state changed)
- …

### Dispatched (fire-and-forget)
- …

### Waiting on others (on the board, not executed)
- …

### Not moved (honest)
- …
```

## Parallelization rules

- **Different repos** → always parallel (separate clones/processes)
- **Same repo, different branches** → worktrees or sequential if one clone
- **protogen** → one at a time per c1 clone (docker lock); start it early in parallel with other repos
- **Linear API** → parallel mutations OK
- **sqfan dispatch** → fire-and-forget; don't poll to completion during scramble

## Pairing

| Need | Skill |
|---|---|
| Which milestone is active | **tactical-sitrep** (before scramble if unclear) |
| c1 dispatch briefs | **c1-squire-dispatch** |
| sqfan mechanics | **sqfan** |
| CI red on blocker PR | **gh-fix-ci** |

## Anti-patterns

- Single-threading when independent options exist
- Planning deck without executing
- Waiting on squire for protogen you can run locally
- Tracker comments during scramble
- sqfan for <15m local work
- Done in tracker while a chain PR is still open
- Ending with "you should run …"

## Project defaults

Latchkey reviewer sets, milestone IDs, repo chains: `references/latchkey.md`.