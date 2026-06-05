---
name: static-analysis-triage
description: Convert output from custom or experimental static analysis tooling into actionable bug fixes. Use when working with novel linters, in-house detectors, or recently-shipped analysis passes whose findings haven't yet been triaged into PRs. Covers detector volume reduction, per-finding source verification, false-positive shape recognition, and end-to-end PR drafting. NOT for established lints (gosec, staticcheck, eslint) where the rules are well-known and the fix is mechanical.
---

# Static Analysis Triage

Captures the methodology for going from raw detector output to merged PRs. The
core claim: candidate counts and bug counts are different numbers, and the
ratio between them depends on the detector, the codebase, and the consumer's
appetite for cosmetic fixes. Don't treat "N candidates" as "N bugs."

## When This Applies

- A new detector just shipped and produced output against a real codebase.
- An in-house tool generates per-function findings with witness paths.
- The user expects PRs against an external repo (not just an issue list).
- The fix per finding is non-mechanical and requires reading the surrounding
  context.

## When This Doesn't Apply

- The tool is a well-established linter with a stable rule set (`go vet`,
  `staticcheck`, `eslint`). Use the tool's own auto-fix or skip it.
- The user only wants metrics or a counts dashboard, not fixes.

## Workflow

### 1. Sanity-check detector volume before diving in

If a detector produces 10x more candidates than its peers, something is off.
Likely causes (ordered by frequency):

- **Generated code** is in the candidate set. Filter on the
  `// Code generated ... DO NOT EDIT` preamble.
- **Vendored code** is in the candidate set. Look for path prefixes:
  `vendor/`, `third_party/`, `build/proto-vendor/`, `local_vendor/`. The exact
  conventions are project-specific.
- The caller / sink predicate is too loose. Tighten before triage; otherwise
  you waste effort reading the same false-positive shape repeatedly.

If the count is small (< 20), skip the volume-reduction step and go straight
to per-finding triage. Tightening predicates against tiny samples just
overfits.

### 2. Pick the smallest-count detector first

Triaging 4 candidates teaches you the false-positive shapes faster than
triaging 4000. Once you know the shapes, you can scan larger detectors for the
same patterns instead of reading each finding line-by-line.

### 3. Per-finding triage

For each finding, read the actual source. Don't trust the heuristic. Classify
into one of:

- **False positive** — the detector fired but the code is correct as written.
  Example: `time.Sleep(100*time.Millisecond)` AFTER the work completes ("let
  the spinner success message render") is irrelevant to ctx because there's
  nothing to cancel.
- **Real but low-impact** — the detector is correct but the user impact is
  small enough that a PR isn't worth the reviewer's time. Example: a Ctrl-C
  delay in a developer CLI tool that only the engineer running it
  experiences.
- **Real bug** — the detector is correct AND the impact justifies a fix.
  Example: a production request handler holds a lock through a slow IO call,
  blocking every other request that needs that lock.

The triage step is the work. There is no shortcut. The detector tells you
where to look; it does not tell you whether to fix.

### 4. Look at immediate call context, not just signatures

A function signature `func DoWork(ctx context.Context)` is necessary but not
sufficient context for triaging a ctx-related finding. The `time.Sleep` call
*site* is what matters: the literal duration, the surrounding statements, the
comments. The same sink can be a false positive in one caller and a real bug
in another.

### 5. Write the PR by hand

Don't generate PRs from a template script. Each fix needs:

- A description that's specific to the bug shape at this location
- A fix that reads the surrounding code (variable names, ctx parameter
  identifier, error-handling style)
- Reviewer assignment per the project's conventions

Auto-drafting at scale destroys reviewer trust the first time it submits a
fix against a false positive. The user explicitly stated this constraint.

### 6. Triage your own asides too

While reading source for a flagged finding, you'll often notice unrelated
issues — "wait, that cache check looks wrong" or "this error caching looks
permanent." Treat those asides with the same skepticism as the original
detector finding: don't dismiss them, but don't assume them either.
Specifically: when the original finding turns out to be a false positive,
don't just stop. The asides are independent observations and may be the
real bugs.

Concrete example: a `lock_held_during_external_call` finding on AWS
`getIdentityInstance` turned out to be a known-acceptable lazy-init
pattern (FP). But while reading the function I noticed:

- The cache hit check uses `== 1`, so multi-instance accounts re-query
  AWS on every call AND grow the cache slice unboundedly (real bug,
  shipped as a 1-line PR).
- The error cache is permanently sticky — one transient failure poisons
  the connector for the activity's lifetime (real but narrow bug,
  shipped as a separate PR with the tradeoff documented).

Both real bugs were invisible to the detector; only careful reading
surfaced them. The detector pointed me at the right function for an
unrelated reason. **The detector is a teleporter, not an oracle**: it
gets you to the right place, but you still have to look around once you
arrive.

When this happens:

- **Verify the aside the same way you'd verify a finding**: read the
  code, check the lifecycle assumptions, confirm the impact is real.
- **One aside per PR**: don't bundle the FP-confirmation with the
  separately-discovered bug. Each PR has one bug shape.
- **Document the discovery path in the PR**: "Found while reviewing X
  detector finding; the original finding is intentional, but this
  separate issue surfaced incidentally" — this gives reviewers the
  context they need without claiming the detector found the actual bug.

## Recurring False-Positive Shapes

These are the patterns to recognise on sight:

### `transitive_ctx_unaware_sleep` / `ctx_unaware_sleep`

- **Cosmetic delays after completion**: `time.Sleep(100*time.Millisecond)`
  after a spinner.Success() call, after an agent's final message, after a
  log.Printf("done"). The work is finished; ctx cancellation has nothing to
  interrupt.
- **CPU-yield / busy-wait mitigation**: very short sleeps (≤10ms) inside a
  polling loop are a different idiom than cancellation gaps. The repo's
  per-function detector already has a 10ms cutoff for this.

### `transitive_ctx_unaware_channel_recv` / `ctx_unaware_channel_recv`

- **Receives from a goroutine the function itself spawned**: if the function
  also does `go func() { ... ch <- ... }()`, the receive is bounded by the
  goroutine's lifecycle, not external state. ctx might still matter for the
  goroutine, but not for the receive.

### `transitive_unscoped_context_background`

- **Background goroutines that intentionally outlive the caller's ctx**:
  detached workers, audit-log flushers, "fire-and-forget" notifications.
  Look for nearby comments mentioning "fresh", "detached", "outlives",
  "deferred", "debounce", "long-lived", "background".
- **Type / schema registration bootstraps**: `RegisterFooObject(tp)`,
  builder-style functions that wire up a TypeProvider, codec registrar,
  CEL environment etc. They run at startup time with no parent ctx in
  scope. Calling `context.Background()` is correct; there's nothing to
  inherit from.
- **Pass-through-but-unused ctx**: a function takes ctx in its
  signature for API uniformity but never uses it (or only uses it for
  log extraction). When such a function calls a downstream helper with
  `context.Background()`, no real ctx is being lost — the upstream
  ctx wasn't doing anything anyway. Check whether ctx is actually
  threaded into IO / cancellation / deadline anywhere in the chain.

### `transitive_reorder_sensitive_function`

- This category is heuristic, not a bug class. The detector tells you a
  caller transitively reaches an ordered region. Whether the caller violates
  ordering depends on what the caller does. Default verdict: needs human
  judgment per finding; don't auto-PR.

### `lock_held_during_external_call`

- **The external call is the lock's purpose**: e.g. a connection pool's
  `Get()` legitimately holds the pool mutex while it does the network
  handshake to a free connection. The lock IS the contention; releasing it
  defeats the pool.
- **Lazy-init under mutex**: cache mutex held through the first call's
  network IO so concurrent callers all see the populated cache. The
  alternatives (release lock during IO, sync.Once, singleflight) either
  cause thundering-herd duplicate IO, or have identical wait behavior.
  The detector flags the pattern correctly but the wait IS the design.
  Don't auto-PR. Worth reading the function for adjacent issues
  though — see "Triage your own asides too" above.
- **Distributed locks**: a `Lock(ctx) error` / `Unlock(ctx)` API
  (Redis-backed mutex, postgres advisory lock, etc.) is held through
  the IO it's protecting BY DESIGN. Holding a workflow lock through
  the workflow's termination call is the entire point. If the
  detector doesn't distinguish distributed locks from `sync.Mutex`,
  every distributed-lock callsite shows up as a finding.
- **Now() / cheap syscalls**: classifiers may tag `time.Now()` as
  `io_sync` because it's technically a syscall. Holding a lock
  through `time.Now()` is fine; the syscall is sub-microsecond.

## Volume Reduction Heuristics

When tightening a detector's caller / sink predicate to reduce noise:

- **Caller predicates that exclude "already in the same world"**: if the
  caller has the same EffectClass as the sink (e.g. both `external_mutation`),
  flag it less aggressively. The cross-fn variant is meant to catch
  cross-boundary entries, not intra-region calls.
- **Skip generated and vendored code categorically** for any cross-fn
  detector that produces > 1000 candidates. The signal-to-noise ratio in
  third-party / generator-imposed code is poor.
- **Don't tighten against small samples**. A detector with 27 candidates
  doesn't need a tighter predicate; just triage each one.

## Reporting Triage Results

For each detector batch, report:

- N total candidates
- N false positives
- N real-but-low-impact (with brief reason)
- N real bugs (with PR links if drafted)

The ratio of "real bugs / total candidates" is the detector's effective
precision on this codebase. Track it over time; if it stays below ~5%, the
detector probably needs further tuning before it's worth running on every
build.

## Related Skills

- `golang-code-review`: when reviewing the candidate-fix code itself before
  PR submission.
- `git-pr`: for the actual PR-creation workflow once the fix is ready.
- `dry-engineering`: PR descriptions stay factual; no superlatives, no
  "improvement" framing.
