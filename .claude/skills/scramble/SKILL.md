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

Time-boxed sprint (default **30 minutes**): build an option board of everything that could move, execute the largest independent set in parallel, locally, then score what changed state. Not sitrep (one goal → A/B/C call), not orient (sweep → gestalt), not one task run end-to-end. Bias to doing: no reports, tracker comments, or planning docs unless asked. If an architectural decision must precede code, plan first — not a scramble.

## Hard rules

1. Options first — scan everything that can move before committing to any single thread.
2. Local action wins — protogen, rebases, tests, pushes, tracker states, batch writes run on host; never wait on squire for mechanical work.
3. Execute yourself — never end with "you should run …".
4. Linear: state transitions only, no comments (unless overridden).
5. Link PRs at every mention.
6. Fan-out limit is independence — options sharing a dirty tree or branch serialize.

## Workflow

**1. Pin the window (30s).** Time budget (default 30m), scope, metric = count of state transitions (merged, pushed, opened, dispatched, tracker state changed).

**2. Option board (2–3 min).** Tracker items in scope + real-world signals (PRs, branches, CI, blockers). One line per option: what, type, Local?, depends-on, estimate. Mark Local? honestly — remote-blocked options stay on the board but never enter the execution set.

| Type | What it is |
|---|---|
| close-in-review | chain ready to land; rebase blocker, re-request review, or Done if merged |
| unblock-in-progress | branches exist; needs protogen, rebase, conflict fix, draft PR |
| fleet-dispatch | Todo item well-scoped for sqfan; write batch + fire |
| tracker-hygiene | wrong state |
| open-PR | branch pushed, no PR yet |
| local-test | targeted tests to unblock a push |
| remote-blocked | needs human review/approval/decision — note, don't pretend |

**3. Plan (1 min).** Execution set: every local, independent, state-moving option; prioritize close-in-review and unblock-in-progress (highest delta-v per minute); 1–3 fleet dispatches if they won't steal focus from local pushes; cap at 3–6 parallel threads. Present as a numbered list (to run, not to approve), then start all independent items immediately.

**4. Execute (bulk of window).**

Close-in-review, per chain: map proto → server → SDK → CLI PRs. All merged → tracker Done. Blocker PR open → rebase on main, fix conflicts, `make protogen`, targeted tests, push, re-request reviewers. Never Done while any chain PR is still open.

Unblock-in-progress: fetch + rebase every repo in the chain; local protogen/tests, fix only what breaks; push, open draft PRs for missing layers; tracker → In Review once the stack is PR-visible.

c1 protogen — always local, one at a time per clone (docker lock), so start it early alongside other repos: merge origin/main, fix .proto by hand (drop duplicate rpc stubs), `make protogen`, targeted `go test`, push.

Fleet-dispatch (Todo → sqfan):

| Signal | Dispatch? |
|---|---|
| Well-scoped, ≤3 repos, clear deliverable | Yes |
| Needs running backend | Yes + `c1-dev-stack` |
| Adjacent parity work on existing branch family | Yes |
| Decision / ADR / "decide whether" | No |
| Large platform / hardware greenfield | No |
| Blocked on in-flight stack merge | No |
| <15 min of local work | No — do it locally |

Write `sqfan-batches/<slug>/batch.yaml` + prompts; dispatch fire-and-forget via sqfan MCP — don't poll to completion during the scramble. Cap 2–3 dispatches per scramble.

Tracker hygiene (throughout):

| Condition | State |
|---|---|
| Stack PR-visible, CI running | In Review |
| All chain PRs merged | Done |
| No work, wrongly In Progress | Todo |
| Decision ticket | Todo |

**5. Scorecard (last 2 min).**

```markdown
## Scramble scorecard — <date> (<N>m)

**Plan:** <N> options assembled, <M> executed in parallel
**Delta-v:** <one sentence>

### Executed
### Landed (state changed)
### Dispatched (fire-and-forget)
### Waiting on others (on the board, not executed)
### Not moved (honest)
```

## Pairing

| Need | Skill |
|---|---|
| Which milestone is active | tactical-sitrep (before scramble if unclear) |
| c1 dispatch briefs | c1-squire-dispatch |
| sqfan mechanics | sqfan |
| CI red on blocker PR | gh-fix-ci |

Project defaults (Latchkey reviewer sets, milestone IDs, repo chains): `references/latchkey.md`.
