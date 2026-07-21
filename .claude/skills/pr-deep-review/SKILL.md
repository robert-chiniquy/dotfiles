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

Two modes:

- **Initial review** — Phases 0–6.
- **Re-review** — author addressed feedback; see [Re-review mode](#re-review-mode).

Read-only on the code — never edit the branch. Writes externally only via
`gh api` to post the review; confirm with the user before posting.

## Phase 0 — Triage and pre-flight

Triage by diff size (fan-out costs 8–12+ agent invocations):

```bash
git diff --shortstat <base>...HEAD
git diff --name-only <base>...HEAD | wc -l
```

| Diff size | Path |
|---|---|
| < 50 changed lines, or only generated / docs / lockfiles | Single-pass review — do not use this skill. Tell the user. |
| 50–1000 lines AND ≤ 20 hand-written files | This skill; skip dimensions the diff doesn't touch. |
| > 1000 lines OR > 20 hand-written files OR cross-module | Full fan-out; treat the architect's probe budget as generous. |

**Pre-flight.** Map each dimension's "reads first" to skills/rules that
actually exist in the *current repo* — not a hardcoded set:

```bash
ls .claude/skills/*/SKILL.md 2>/dev/null
ls .claude/rules/*.md 2>/dev/null
```

If a dimension has no repo-local skill, the agent runs on its lane definition
alone; note the gap in the review summary and suggest authoring one.

**Generated-file exclusion.** Per-repo. Start from: `*.pb.go`, `*_pb.ts`,
`*.pb.cc`, `*.pb.h`, `pb/**`, `wire_gen.go`, `*_gen.go`,
`openapi.{yaml,json}`, `*.oas31.yaml`, `__generated__/**`, `*.snap`,
`package-lock.json`, `pnpm-lock.yaml`, `go.sum`, `Cargo.lock`,
`*.min.{js,css}`. Narrow with the repo's `.gitignore` and project-specific
lists (e.g. c1's OPA `data.json` bundles, occult's generated solver tables).

## Phase 1 — Scope the diff

1. Locate the PR: `gh pr view --json number,title,url,state` or
   `gh pr list --head <branch> --state all --json number,url,state`.
2. `git log --oneline <base>..HEAD`; `git diff --stat <base>...HEAD`;
   `git diff --name-only <base>...HEAD`.
3. Subtract the generated-file set; the remainder is the review surface.
4. Write a one-paragraph summary of the PR. Every dimension agent gets it.

## Phase 2 — Research: one agent per dimension

Launch applicable dimensions in parallel, in a single message. Research, not
verdicts: each agent reports observations in its lane only; the architect
(Phase 3) arbitrates severity and cross-dimension calls.

Give each agent: branch, base ref, the summary, the hand-written file list,
the repo-local skills mapped in Phase 0, and the standing rules below.

| Dimension | Reads first (repo-local, if present) | Hunts for |
|---|---|---|
| **Security** | `security-patterns`, `.claude/rules/security.md`, OWASP cheat sheets if no repo skill | authz gating, tenant isolation, input validation, caller-identity spoofing, error-detail leakage, fail-open defaults, secret exposure in diffs, new external deps with credential scope |
| **Scale** | `pgdb-index-coverage`, `temporal-workflows`, or repo-local SQL/index conventions | unbounded queries, missing indexes, N+1 / per-row loops, large in-memory loads, workflow fan-out, "works at 100 users, breaks at 1M" |
| **Performance** | `postgres-query-perf` or repo-local perf rules | redundant DB calls per request, blocking I/O on hot path, repeated work in loops, recomputed schema/form parsing, needless (de)serialization |
| **Correctness** | `go-conventions` / `rust-conventions` / language-specific repo rules | nil/empty handling, swallowed errors, edge cases, race conditions, proto field semantics, **test-coverage gaps** — do tests assert the gRPC status *code*, not just non-nil? failure paths or only happy path? |
| **Idiomatic style** | `go-conventions`, `.claude/rules/{backend,frontend,comments}.md`, language style guides | **Reinvented helpers** — hand-rolled map/filter/dedup/pagination/retry/hashing an existing repo utility provides; logging not via repo conventions; gRPC error *codes* misclassified; comment smells; functions that need paragraphs instead of a rename |
| **Frontend** (only when diff is ≥30% frontend files) | `react-patterns`, `.claude/rules/frontend.md` | a11y, state-management coupling, render perf, hydration mismatches, type-narrowing escape hatches, prop-drilling vs. context boundaries |

**Standing rules in every dimension agent's prompt:**

- Scope **strictly** to `git diff <base>...HEAD`. Per observation:
  `file:line`, what you see, why it matters, a concrete fix, and a
  *provisional* severity (critical/high/medium/low) — architect arbitrates.
- **Do NOT make code changes.** Read-only.
- Be specific to this diff; skip generic advice.
- Flag patterns that are **pre-existing on `<base>`** rather than introduced
  by this branch.
- **Cap findings at ~15 per dimension.** If over, return the 15
  highest-confidence and flag "lane is finding-saturated, suggest a followup
  pass."

## Phase 3 — Architect: collate the dimensional research

One architect agent holds all dimension outputs at once:

1. **Dedups.** Collapse the same line flagged from multiple angles (e.g.
   "unindexed" + "slow") into one finding naming every angle.
2. **Finds the seams** where two dimensions meet (e.g. a missing index only
   pathological because an authz filter forces a full scan; a swallowed error
   that is a fail-open). One targeted probe agent per seam worth chasing.
3. **Caps probes at 3 total.** Excess seams become Tier-3 design questions in
   Phase 5. One hop, not a loop.
4. **Arbitrates severity** across lanes.
5. **Emits one ranked candidate list**: `file:line`, dimensions touched,
   provisional severity, provisional in-scope/pre-existing guess,
   recommended fix.

The architect does not post and does not make final tier calls — everything
flows through Phase 4.

## Phase 4 — Adversarially verify every finding

**Do not skip.** Spawn verifier agents that try to **refute** each candidate.
Batching:

- ≤ 6 findings → one verifier per finding
- \> 6 → batch by area: security/style, scale/perf, correctness

For idiomatic-style findings, the verifier must confirm the claimed helper
actually exists and fits the use before the finding stands.

Verdict per finding — **TRUE / FALSE / PARTIALLY TRUE**, backed by the actual
code — answering:

1. **Does the code support the claim?** Wrong substance = FALSE. Wrong line,
   right substance = PARTIALLY TRUE: do **one re-location hop** — find the
   real instance, re-verify in place, carry forward as TRUE at the correct
   `file:line`. One retry only; still unlocatable = FALSE. Don't silently
   drop a real defect over a mis-citation.
2. **Introduced by this branch, or pre-existing?**
   ```bash
   git diff <base>...HEAD -- <file>
   git log <base>..HEAD -- <file>
   git blame -L<line>,<line> <file>
   ```
   Pre-existing = out of scope.
3. **Severity justified or inflated?** e.g. HIGH on a path that
   short-circuits on first match, or runs once per infrequent activity, is
   usually inflated.

Prompt verifiers to **default to skepticism** — a finding stands only if the
code supports it.

## Phase 5 — Synthesize and tier (orchestrator — you)

Drop everything refuted. Tier what survives:

1. **Blocking & in-scope** — real defects or missing tests in *this PR's*
   new code.
2. **Cheap in-branch nits** — small, non-blocking. Batch them.
3. **Design questions** — confirmed-true judgment calls. Phrase as questions.
4. **Pre-existing / out-of-scope** — mention, propose a follow-up ticket,
   explicitly say "do not fix in this PR."

Never ask the author to fix pre-existing code in a feature PR; never post a
finding the verifier refuted.

Idiomatic-style findings default to Tier 2 / non-blocking. Exception: a
reinvented helper the repo explicitly bans rebuilding (an authoritative "USE
THESE, DON'T REWRITE THEM" list) is a real defect and can be Tier 1.

## Phase 6 — Shape for an agent consumer, then post

Assume the author hands the review to *their own agent*:

- **Anchor to the specific line.** Inline comments attach only to
  added/changed lines (`side: RIGHT`); findings on lines outside the diff
  hunks go in the summary body.
- **Self-contained + exact**: the precise change wanted (or that it's a
  question), enough context to act without re-deriving, and a verification
  step (e.g. `make test/pkg PKGS=...`).
- **Scope guards in two places** (agents dutifully "fix" everything
  mentioned):
  1. Summary body: a **"do NOT change these"** block listing refuted and
     pre-existing/out-of-scope items.
  2. Any inline comment on a pre-existing pattern: prefix
     `[pre-existing — do not change in this PR]` — agents read inline more
     reliably than summary.

Pick the event and **confirm with the user before posting**:

- `COMMENT` — default for a first pass with real asks
- `REQUEST_CHANGES` — genuine blockers only
- `APPROVE` — clean / re-review pass

Write the payload to a JSON file, then:

```bash
gh api repos/<owner>/<repo>/pulls/<number>/reviews \
  --method POST \
  --input <payload>.json \
  --jq '{state: .state, html_url: .html_url}'
```

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

Block-level comments use `start_line` + `line` + `start_side` + `side`.

## Re-review mode

1. **Find the delta.** `git log --oneline <base>..HEAD`,
   `git show <fix-commit>`, plus prior threads and author replies:
   ```bash
   gh api repos/<owner>/<repo>/pulls/<n>/comments
   gh pr view <n> --json comments
   ```
2. **Run the same process on the delta**, steering dimension agents to:
   (a) confirm each prior finding correctly + completely resolved;
   (b) hunt **regressions introduced by the fixes** — a fix that tightens a
   shared predicate or changes a gate is the riskiest delta; trace whether it
   can break in-flight / legacy state. Don't re-report items already ruled
   out of scope.
3. **Verify new findings** (Phase 4) — especially any "the fix is safe"
   assertion: trace *why* it's safe, don't take the agent's word.
4. If clean, `APPROVE` with a body confirming each item resolved and
   recording *why* the risky changes are safe. Remaining nits explicitly
   optional / out-of-scope.
