---
name: pr-deep-review
description: >-
  Deep, multi-agent review of a PR or branch diff: fan out one focused subagent
  per dimension (security, scale, performance, correctness, idiomatic style,
  plus frontend when the diff warrants), adversarially verify every finding to
  kill false positives and pre-existing debt, tier what survives, then post
  agent-shaped inline comments to the PR. Has a re-review mode for when the
  author has addressed feedback. Use when the user asks to "deep review this
  branch/PR", "review the PR with subagents", "do a thorough review",
  "re-review the PR", or wants a higher-rigor pass than a single-shot review.
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
  - Agent
---

# Deep PR review

A higher-rigor alternative to single-pass review. The value is in two things a
single pass doesn't do: **one specialist per dimension** (no single agent's
attention is split across security AND scale AND correctness), and an
**adversarial verification pass** that refutes findings before they reach the
author. The verification step is what makes this trustworthy — in practice it
kills false positives, demotes inflated severities, and catches the most common
failure of automated review: flagging *pre-existing* code that the PR didn't
touch.

Two modes:

- **Initial review** — first deep pass on a branch/PR (Phases 0–6 below).
- **Re-review** — author addressed feedback; confirm fixes, hunt regressions
  introduced by the fixes, approve. See [Re-review mode](#re-review-mode).

This skill is read-only on the code. It does NOT edit the branch. It does write
externally via `gh api` to post the review — confirm with the user before
posting.

---

## Phase 0 — Triage and pre-flight

**Triage by diff size.** Fan-out is expensive (potentially 8–12+ agent
invocations). Use the cheapest tool that suffices:

```bash
git diff --shortstat <base>...HEAD
git diff --name-only <base>...HEAD | wc -l
```

| Diff size | Path |
|---|---|
| < 50 changed lines, or only touches generated / docs / lockfiles | Single-pass review (do not invoke this skill). Tell the user. |
| 50–1000 lines AND ≤ 20 hand-written files | This skill, but skip dimensions the diff doesn't touch (e.g. no Scale agent for a frontend-only PR; no Frontend lane for a backend-only PR). |
| > 1000 lines OR > 20 hand-written files OR cross-module | Full fan-out; treat the architect's probe budget as generous. |

**Pre-flight check.** Verify the per-dimension "read first" skills exist in the
*current repo* (look under `.claude/skills/`, `.claude/rules/`, or the
project-root convention the repo uses). Build the actual list before fan-out:

```bash
ls .claude/skills/*/SKILL.md 2>/dev/null
ls .claude/rules/*.md 2>/dev/null
```

Map each dimension to the **specific repo-local skills/rules that actually
exist**, not a hardcoded set. If a dimension has no useful repo-local skill,
the agent still runs against its general lane definition (below) — but flag in
the review summary that the repo lacks a `<dimension>-patterns` reference and
suggest authoring one.

**Generated-file exclusion.** Build this list per-repo, not hardcoded. Start
from common globs: `*.pb.go`, `*_pb.ts`, `*.pb.cc`, `*.pb.h`, `pb/**`,
`wire_gen.go`, `*_gen.go`, `openapi.{yaml,json}`, `*.oas31.yaml`,
`__generated__/**`, `*.snap`, `package-lock.json`, `pnpm-lock.yaml`, `go.sum`,
`Cargo.lock`, `*.min.{js,css}`. Then narrow with the repo's own `.gitignore`
and any project-specific list (e.g. c1's OPA `data.json` bundles, occult's
generated solver tables).

---

## Phase 1 — Scope the diff

1. Locate the PR: `gh pr view --json number,title,url,state` or
   `gh pr list --head <branch> --state all --json number,url,state`.
2. Pull diff + file list:
   ```bash
   git log --oneline <base>..HEAD
   git diff --stat <base>...HEAD
   git diff --name-only <base>...HEAD
   ```
3. Subtract the generated-file set from Phase 0. The remaining hand-written
   files are the review surface.
4. Write a one-paragraph summary of what the PR does. **Every dimension agent
   gets this** so they don't each re-derive it.

---

## Phase 2 — Research: one agent per dimension

**Research phase, not verdict phase.** Launch the applicable dimensions **in
parallel, in a single message**. Each owns exactly one dimension and reports
what it *observes* in its lane — it does not decide final severity or whether
a finding blocks. Cross-dimension arbitration is the architect's job (Phase 3).

Give each agent: the branch, the base ref, the one-paragraph summary, the
hand-written file list, the repo-local skills mapped in Phase 0 (none if no
repo-local skill exists), and the standing rules below.

| Dimension | Reads first (repo-local, if present) | Hunts for |
|---|---|---|
| **Security** | `security-patterns`, `.claude/rules/security.md`, OWASP cheat sheets if no repo skill | authz gating, tenant isolation, input validation, caller-identity spoofing, error-detail leakage, fail-open defaults, secret exposure in diffs, new external deps with credential scope |
| **Scale** | `pgdb-index-coverage`, `temporal-workflows`, or repo-local SQL/index conventions | unbounded queries, missing indexes, N+1 / per-row loops, large in-memory loads, workflow fan-out, "works at 100 users, breaks at 1M" |
| **Performance** | `postgres-query-perf` or repo-local perf rules | redundant DB calls per request, blocking I/O on hot path, repeated work in loops, recomputed schema/form parsing, needless (de)serialization |
| **Correctness** | `go-conventions` / `rust-conventions` / language-specific repo rules | nil/empty handling, swallowed errors, edge cases, race conditions, proto field semantics, **test-coverage gaps** — do tests assert the gRPC status *code*, not just non-nil? do they cover failure paths or only happy path? |
| **Idiomatic style** | `go-conventions`, `.claude/rules/{backend,frontend,comments}.md`, language style guides | **Reinvented helpers** — hand-rolled map/filter/dedup/pagination/retry/hashing that an existing repo utility already provides; logging not via repo conventions; gRPC error *codes* misclassified; comment smells (storytelling, restating code); functions that need paragraphs to explain instead of a rename |
| **Frontend** (only when diff is ≥30% frontend files) | `react-patterns`, `.claude/rules/frontend.md` | a11y, state-management coupling, render perf, hydration mismatches, type-narrowing escape hatches, prop-drilling vs. context boundaries |

**Standing rules in every dimension agent's prompt:**

- Scope **strictly** to `git diff <base>...HEAD`. For each observation report
  `file:line`, what you see, why it matters, a concrete recommended fix, and a
  *provisional* severity (critical/high/medium/low). Provisional — architect
  arbitrates.
- **Do NOT make code changes.** Read-only.
- Be specific to this diff; skip generic advice.
- Note when a flagged pattern is **pre-existing on `<base>`** rather than
  introduced by this branch (verifier will double-check, but flag it).
- **Cap your findings at ~15 per dimension.** If your lane has more, return
  the 15 highest-confidence and flag "lane is finding-saturated, suggest a
  followup pass."

---

## Phase 3 — Architect: collate the dimensional research

A single architect agent holds **all dimension outputs at once** and turns raw
evidence into one coherent candidate list. The only place cross-cutting
judgment happens. The architect:

1. **Dedups.** Lanes routinely flag the same line from different angles — a
   query is both "unindexed" (scale) and "slow" (performance). Collapse into
   one finding that names every angle, so the verifier refutes it once and the
   author reads it once.
2. **Finds the seams.** The nastiest defects live where two dimensions meet:
   a missing index (scale) that's only pathological because an authz filter
   (security) forces a full scan; a swallowed error (correctness) that's
   actually a fail-open (security); an unbounded load (scale) behind a hot path
   (performance). For each seam worth chasing, fire **one targeted probe** —
   a focused agent scoped to just that seam, not a re-run of a lane.
3. **Caps the probe budget at 3 total.** If the diff has more candidate seams
   than 3 probes can cover, tier the remainder as Tier-3 design questions in
   Phase 5 rather than firing more probes. One hop, not a loop, and bounded
   total cost.
4. **Arbitrates severity.** Rank across dimensions and set a single
   provisional severity per finding — a security agent over-ranks security,
   a perf agent over-ranks perf; the architect, seeing all lanes, calibrates.
5. **Emits one ranked candidate list**, each item carrying `file:line`, the
   dimensions it touches, provisional severity, provisional
   in-scope/pre-existing guess, and the recommended fix.

The architect does **not** post and does **not** make the final tier call —
its severities and scope guesses are still *provisional*. Everything flows
through Phase 4, the actual gate.

---

## Phase 4 — Adversarially verify every finding

This is the step that earns trust. **Do not skip it.** Take the architect's
deduped candidate list, then spawn verifier agents that try to **refute** each
claim. Batching rule:

- ≤ 6 candidate findings → one verifier per finding
- > 6 findings → batch by area: one verifier for security/style,
  one for scale/perf, one for correctness, fan out as needed

For idiomatic-style findings, the verifier must confirm the claimed helper
*actually exists and fits* (e.g. that `pkg/uslice` really has the function,
and the hand-rolled code isn't doing something the helper can't) before the
finding stands — "you could use a helper" that doesn't apply is a false
positive.

Each verifier, per finding, returns a verdict **TRUE / FALSE / PARTIALLY TRUE**
backed by the actual code, and answers three questions:

1. **Does the code actually support the claim?** Read the cited lines. Wrong
   line number with right substance is "partially true"; wrong substance is
   "false." **On PARTIALLY TRUE for a wrong-location-but-real finding, do one
   re-location hop:** have the verifier find the actual instance and re-verify
   it in place, then carry forward as TRUE anchored to the correct
   `file:line`. Don't silently drop or downgrade a real defect just because
   the finder mis-cited it. **One retry only** — if it still can't be located,
   it's FALSE.
2. **Did this branch introduce it, or is it pre-existing?**
   ```bash
   git diff <base>...HEAD -- <file>
   git log <base>..HEAD -- <file>
   git blame -L<line>,<line> <file>
   ```
   A real pattern that predates the branch is **out of scope** — the PR didn't
   cause it and shouldn't be forced to fix it.
3. **Is the severity justified or inflated?** "HIGH" on a path that
   short-circuits on first match, or runs once per infrequent activity, is
   usually inflated.

Prompt verifiers to **default to skepticism** — a finding only stands if the
code supports it. This pass routinely converts a scary-looking "REQUEST
CHANGES" list into a couple of real asks plus a pile of refuted/pre-existing
noise.

---

## Phase 5 — Synthesize and tier (orchestrator)

The orchestrator (the agent running this skill — i.e., you) does Phase 5.
Drop everything the verifier refuted. Tier what survives so the author sees
the real asks first, not a flat wall:

1. **Blocking & in-scope** — real defects or missing tests in *this PR's* new
   code. The actual asks.
2. **Cheap in-branch nits** — small, in the new code, non-blocking. Batch them.
3. **Design questions** — confirmed-true but a judgment call (e.g. authz
   floor). Phrase as a question, not a change request.
4. **Pre-existing / out-of-scope** — real but not this branch's doing.
   Mention, propose a follow-up ticket, and **explicitly say "do not fix in
   this PR."**

**The discipline that matters: never ask the author to fix pre-existing code
in a feature PR, and never post a finding the verifier refuted.**

Idiomatic-style findings almost always belong in Tier 2 (cheap nits) or as
non-blocking suggestions — surface them, don't `REQUEST_CHANGES` over them.
The exception is a reinvented helper that the repo explicitly bans rebuilding
(an authoritative "USE THESE, DON'T REWRITE THEM" list in backend rules),
which is a real defect and can be Tier 1.

---

## Phase 6 — Shape for an agent consumer, then post

Assume the author will hand your review straight to *their own agent*. Shape
every comment accordingly:

- **Anchor to the specific line**, not one summary blob. Confirm the line is
  part of the diff first — inline comments can only attach to added/changed
  lines (`side: RIGHT`). Pre-existing lines outside the diff hunks can't take
  an inline comment; put those in the summary body instead.
- **Self-contained + exact**: state the precise change wanted (or that it's
  a question with no change), enough context that the agent needn't
  re-derive, and a **verification step** (e.g. `make test/pkg PKGS=...`).
- **Scope guards in two places** — defense in depth against the well-known
  "the author's agent dutifully fixes everything mentioned" failure:
  1. In the summary body, add a **"do NOT change these"** block listing
     refuted findings and pre-existing/out-of-scope items.
  2. On any inline pre-existing comment that *does* appear (because the line
     happens to be in a diff hunk), prefix the body with `[pre-existing — do
     not change in this PR]` — agents read inline more reliably than summary.

Pick the review event deliberately and **confirm with the user before
posting** (outward-facing; notifies the author):

- `COMMENT` — collaborative, non-blocking; default for a first pass with real
  asks
- `REQUEST_CHANGES` — blocks; reserve for genuine blockers
- `APPROVE` — clean / re-review pass

Post as one review via `gh api`. Write the payload to a JSON file, then:

```bash
gh api repos/<owner>/<repo>/pulls/<number>/reviews \
  --method POST \
  --input <payload>.json \
  --jq '{state: .state, html_url: .html_url}'
```

Payload shape:

```json
{
  "event": "COMMENT",
  "body": "## Review summary ...\n\n**Please address:** ...\n\n**Out of scope — do NOT change these in this PR:** ...",
  "comments": [
    {
      "path": "pkg/api/.../file.go",
      "line": 143,
      "side": "RIGHT",
      "body": "Exact instruction + why + verify step."
    }
  ]
}
```

Multi-line comments use `start_line` + `line` + `start_side` + `side`. Use
those when the finding is about a block (e.g. an entire function), not a
single line.

---

## Re-review mode

When the author says they've addressed feedback:

1. **Find the delta.** `git log --oneline <base>..HEAD` to spot the new fix
   commit(s); `git show <fix-commit>` to see exactly what changed. Pull the
   PR review threads and author replies:
   ```bash
   gh api repos/<owner>/<repo>/pulls/<n>/comments
   gh pr view <n> --json comments
   ```
2. **Run the same process on the delta**, but steer the dimension agents to:
   (a) confirm each prior finding is correctly + completely resolved, and
   (b) hunt for **regressions introduced by the fixes** — a fix that tightens
   a shared predicate or changes a gate is the riskiest delta; trace whether
   it can break in-flight / legacy state. Tell agents not to re-report items
   already ruled out of scope.
3. **Verify the new findings** (Phase 4) — especially any "the fix is safe"
   assertion. Trace *why* it's safe (e.g. "an empty value can't reach this
   gate because creation enforces it"), don't just take the agent's word.
4. **Post the outcome.** If clean, `APPROVE` with a body that confirms each
   item resolved and — usefully for the author's agent — records *why* the
   risky changes are safe. Keep any remaining nits explicitly optional /
   out-of-scope.

---

## Principles (why this beats a single-pass review)

- **One specialist per dimension** — undivided attention finds more than one
  generalist pass. Specialists *research* their lane; they don't render
  verdicts.
- **One architect over all dimensions** — the same logic applied to the
  cross-cutting work: a single agent that holds all lanes dedups overlaps,
  finds the seams where the worst defects live, and arbitrates severity that
  no lane-bound specialist can rank fairly. Synthesizes once — not an open
  loop.
- **Refute before you report** — verification is the quality gate; it removes
  false positives and pre-existing-debt noise that erode author trust. A
  real-but-mis-located finding gets one re-location hop, not a silent drop.
- **Pre-existing ≠ this PR's problem** — flag it, ticket it, don't block on
  it.
- **Write for the next agent** — precise anchors, scope guards in two places,
  verify steps, so the author's agent can act without re-deriving your
  reasoning.
- **Bound the cost** — Phase 0 triage, ~15-finding lane cap, ≤3 probes,
  verifier-batches above 6 findings. The methodology is good; over-applied,
  it becomes its own opposite.
- **Confirm before posting** — a posted review is outward-facing and pings
  a teammate; let the user choose the event type.

---

## Status

**v0.1** (2026-06-18) — initial draft.
