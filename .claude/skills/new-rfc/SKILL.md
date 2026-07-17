---
name: new-rfc
description: >-
  Produce a rigorous, adversarially-reviewed RFC (build-plan / design doc)
  through a multi-phase pipeline — investigate → judges → plan → judges
  (loop until clean) → owner approval. Stops at the approved RFC; does NOT
  dispatch implementation agents or open PRs. Use when the user says
  "write an RFC for X", "build-plan for X", "design doc for X", "produce
  a plan I can approve for X", or wants an adversarially-reviewed RFC
  without the implementation loop.
allowed-tools:
  - Read
  - Write
  - Edit
  - Grep
  - Glob
  - Bash
  - Agent
  - AskUserQuestion
  - TaskCreate
  - TaskUpdate
---

# new-rfc — adversarially-reviewed RFC pipeline

Single-purpose orchestrator: takes a work reference (Linear URL, GitHub
issue, one-line problem statement) and produces a plan document that has
survived at least one round of judge-review. No implementation, no PR
open — the final artifact is the approved RFC.

## Phase graph

```
investigate → judge×N (round 1) → plan → judge×N (round 2)
   → (revise → re-judge)* until clean → 🚦 owner approval → DONE
```

The two judge rounds are the load-bearing rigor. Round 1 surfaces
unknowns / dark corners the plan must address. Round 2 adversarially
validates the plan and drives revisions until no blocking findings.

## Directory layout

Every RFC lives in its own subdir. Choose one of:
- `~/repo/rfcs/plans/<slug>/` — **default** for RFCs meant to be published
  to the private `rfcs` github repo. Standalone, versioned, shareable.
- `<repo>/plans/<slug>/` — when the RFC is intrinsically tied to a
  specific repo's code and shouldn't outlive that repo.
- `~/repo/research/<domain>/<slug>/` — for cross-repo research / exploratory
  RFCs that are local-only (never published; per the global rule
  "Never commit research to git unless the user explicitly instructs it").

Inside the subdir, numbered phase files are durable artifacts. The pattern:

```
plans/<slug>/
├── 00-plan.md                       # meta-plan: phase graph + slug + owner
├── 01-investigation.md              # investigate deliverable
├── 01-judges/                       # round-1 findings, one file per judge
│   ├── correctness.md
│   ├── scale.md
│   ├── security.md
│   └── risk.md
├── 02-plan.md                       # the RFC itself (may be plan-v1, v2, ...)
├── 02-judges/                       # round-2 findings, one file per judge
│   ├── correctness.md
│   ├── ...
├── 02-plan-v2.md                    # if revised after judges
├── 02-judges-v2/                    # judges on v2
└── <slug>-rfc.md                    # symlink or copy of the approved plan
```

## Judge lenses (default: 4)

Distinct viewpoints. Each judge is a separate `Agent` subagent with a
lens-specific prompt. Judges do NOT know each other's outputs — parallel,
independent perspectives. Default set (override at start):

| Lens | Hunts for |
|---|---|
| **correctness** | Does the plan solve the stated problem? Wrong invariants, missed cases, silent-fail paths, spec/implementation mismatches. |
| **scale** | Does it work at production data volumes? Unbounded queries, N+1 loops, per-row RPCs, workflow fan-out that explodes. |
| **security** | Authz gaps, tenant isolation holes, secret exposure, trust-boundary violations, fail-open defaults, confused-deputy paths. |
| **risk** | Backward compatibility, migration hazards, rollout order, blast radius, what breaks if this ships wrong. |

For frontend-heavy RFCs, swap `scale` for **UX-integrity** (a11y, state
coupling, hydration, render perf). For infra-heavy RFCs, swap `security`
for **operability** (observability, on-call surface, failure modes).

## Non-negotiable RFC elements

Every produced plan MUST include, in order:

1. **Header frontmatter** with `harness: claude`, `model: <current>`,
   and status (`ROUND 1 for adversarial review (YYYY-MM-DD)` on first
   emission).

2. **Grounding block** immediately after the title. Every external fact
   is cited to a repo + SHA + verification date:
   ```
   **Grounding (all verified YYYY-MM-DD):** <repo-a> working tree;
   <repo-b> @ <sha> (post-#N/#M); PR #<n> (<subject>) MERGED /
   OPEN / DRAFT.
   ```
   No unverified claims. If a claim can't be grounded, remove it
   or explicitly mark it as an ask.

3. **Status-change table** when this RFC succeeds an earlier canonical
   one. Format:
   ```
   | Canonical-RFC node | Then | Now | Consequence |
   |---|---|---|---|
   | <node> | <prior state> | <current state> | <what changes> |
   ```

4. **Macro outcomes** (O1, O2, …) — user-facing "I can X" statements,
   each paired with a "(Today: <current failure mode>)" contrast that
   cites concrete file:line evidence.

5. **Concrete contracts / interfaces** — proto definitions, function
   signatures, table schemas in the codebase's actual idiom, not
   pseudo-code.

6. **Dependency-ordered build steps** — the reader can execute
   without re-reading the diagnosis. If the WHY is in an earlier
   phase file, incorporate by reference: "the WHY is there and is
   not re-argued here."

7. **No follow-up sections** in the RFC body. Future work belongs in
   separate tracking; the RFC is the executable plan for what ships
   under this document.

## Orchestration protocol

1. **Gather the work ref.** Ask the owner if unclear:
   - What is the deliverable exactly? (One sentence.)
   - Where does the RFC live? (Repo + subdir, or research path.)
   - Are there prior canonical RFCs this succeeds? (For the status
     table.)
   - Any lens swaps from the default judge set?

2. **Bootstrap the phase dir.** Create `plans/<slug>/00-plan.md` with
   the phase graph + owner + slug. This is the durable anchor.

3. **Dispatch investigate.** ONE `Agent` subagent with a phase-1 prompt.
   Deliverable: `01-investigation.md`. The prompt describes ONLY phase 1
   — no downstream orchestration.

4. **Dispatch round-1 judges.** N `Agent` subagents in a SINGLE tool
   message (parallel). Each gets a lens-specific prompt + the
   investigation file. Each writes `01-judges/<lens>.md`.

5. **Synthesize into plan.** Read all judge findings + the
   investigation. Write `02-plan.md` (or dispatch a synth subagent).
   Include all mandatory elements above.

6. **Dispatch round-2 judges.** N `Agent` subagents in parallel on the
   plan. Each writes `02-judges/<lens>.md`.

7. **Assess verdict.**
   - If judges surface blocking findings → revise: write
     `02-plan-v2.md`, then `02-judges-v2/` round. Loop until clean.
   - If judges surface non-blocking notes → include an "unresolved
     but non-blocking" section in the plan; proceed.
   - If clean → present to owner for approval.

8. **🚦 Owner approval gate.** Show the owner:
   - Path to the plan file
   - Judge verdict summary (one bullet per lens: blocking / notes / clean)
   - Any unresolved-but-non-blocking items
   Then **stop**. Do NOT dispatch impl. Do NOT open a PR. The skill's
   deliverable is the approved plan file.

9. **Symlink or copy to `<slug>-rfc.md`** in the phase dir on approval,
   so the canonical filename is stable regardless of how many revisions
   happened.

## Args

| Arg | Meaning |
|---|---|
| `--work-ref <ref>` | Linear URL, GitHub issue, or `requests/<slug>.md` file, or a one-line problem. |
| `--dir <path>` | Where the phase dir lives. Default: prompt the owner. |
| `--judges <n>` | Judges per round. Default 4. Pushing higher costs more but tightens findings; going lower risks blind spots. |
| `--lenses <list>` | Override the default lens set. Comma-separated. |
| `--succeeds <path>` | Path to a canonical RFC this succeeds. Triggers the status-change table. |
| `--resume` | Continue from the latest phase in an existing dir. |

## Judge prompt template

Each judge subagent gets a prompt shaped roughly like:

```
You are the <LENS> judge for RFC <slug>.

The investigation is at <path>/01-investigation.md.
[On round 2: the plan is at <path>/02-plan.md.]

Your job: find <LENS>-shaped problems this document is missing or
underweighting. Distinct lens-specific hunts (see new-rfc lens
table). You do NOT know what other judges are surfacing — bring an
independent view.

Standing rules:
- Scope strictly to the artifact plus its cited grounding files.
- For each finding: file:line, what you see, why it matters, a concrete
  fix, severity (critical/high/medium/low). Severity is provisional —
  the synth step arbitrates.
- Cap at ~10 findings. If more, return the 10 highest-confidence and
  flag "lane is finding-saturated".
- Do NOT edit code or the RFC. Read-only.

Write your findings to <path>/<phase>-judges/<lens>.md. Do not print
them to stdout — the synth step reads the file.
```

## Common mistakes

- **Skipping round 1.** The investigation → judge pass is what surfaces
  the WHY the plan needs to be shaped a specific way. Going straight to
  plan-write loses that.
- **Overloading one judge.** One agent with a "review from every angle"
  brief splits its attention and returns generic findings. Four agents
  with sharp lenses beat one with wide scope.
- **Judges seeing each other's outputs.** Round-N judges must be
  parallel + blind. Serial or shared-context judges converge on the
  same top-level surface finding and miss the tail.
- **Grounding drift.** A plan that cites a SHA which has since moved
  is a plan grounded in fiction. Re-verify grounding on each revision.
- **Proceeding past a blocking finding to "keep momentum".** Blocking
  means blocking. Revise the plan; do not paper over.
- **Auto-advancing past the owner gate.** The skill's contract is that
  the owner approves the plan. Any auto-dispatch to impl violates
  that contract.
