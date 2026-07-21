---
name: static-analysis-triage
description: Convert output from custom or experimental static analysis tooling into actionable bug fixes. Use when working with novel linters, in-house detectors, or recently-shipped analysis passes whose findings haven't yet been triaged into PRs. Covers detector volume reduction, per-finding source verification, false-positive shape recognition, and end-to-end PR drafting. NOT for established lints (gosec, staticcheck, eslint) where the rules are well-known and the fix is mechanical.
---

# Static Analysis Triage

Candidate counts and bug counts are different numbers; the ratio depends on the detector, the codebase, and appetite for cosmetic fixes.

## Workflow

1. **Sanity-check volume first.** A detector producing 10x its peers usually has: generated code in the set (filter on the `// Code generated ... DO NOT EDIT` preamble), vendored code (`vendor/`, `third_party/`, `build/proto-vendor/`, `local_vendor/` — conventions are project-specific), or a too-loose caller/sink predicate. Under ~20 candidates, skip volume reduction and triage each one — tightening predicates against tiny samples overfits.
2. **Triage the smallest-count detector first.** It teaches the false-positive shapes cheaply; then scan larger detectors for those shapes instead of reading each finding line-by-line.
3. **Per finding, read the source and classify**: false positive / real-but-low-impact (correct but not worth reviewer time, e.g. a Ctrl-C delay in a dev-only CLI) / real bug. Judge at the call site — literal durations, surrounding statements, comments — not the signature; the same sink can be an FP in one caller and a bug in another.
4. **Write each PR by hand.** No template scripts — auto-drafting a fix against a false positive destroys reviewer trust (explicit user constraint). Description specific to this location's bug shape; fix matches surrounding style (variable names, ctx identifier, error handling); reviewer assignment per project convention; one bug shape per PR.
5. **Triage your own asides.** Issues noticed while reading flagged code are independent observations — verify them like findings, and don't stop when the original finding is an FP (an FP lock finding has yielded two real cache bugs the detector couldn't see). One aside per PR, and state the discovery path in the PR: "found while reviewing X detector finding; the original finding is intentional; this surfaced incidentally."

## Recurring False-Positive Shapes

### `ctx_unaware_sleep` (incl. transitive)

- Cosmetic delays after the work completes: `time.Sleep(100*time.Millisecond)` after `spinner.Success()`, a final message, a "done" log — nothing left to cancel.
- CPU-yield sleeps ≤10ms in polling loops are a different idiom than cancellation gaps; the per-function detector already has a 10ms cutoff.

### `ctx_unaware_channel_recv` (incl. transitive)

- Receive from a goroutine the function itself spawned: bounded by that goroutine's lifecycle, not external state. ctx may matter for the goroutine, not the receive.

### `transitive_unscoped_context_background`

- Background goroutines meant to outlive the caller's ctx: detached workers, audit-log flushers, fire-and-forget. Nearby comments say "fresh", "detached", "outlives", "deferred", "debounce", "long-lived", "background".
- Startup registration bootstraps (`RegisterFooObject(tp)`, TypeProvider/codec/CEL wiring): no parent ctx exists; `context.Background()` is correct.
- Pass-through-but-unused ctx: signature takes ctx for API uniformity but never threads it into IO/cancellation/deadline (log extraction only), so calling downstream with `context.Background()` loses nothing. Check the whole chain.

### `transitive_reorder_sensitive_function`

- Heuristic, not a bug class: the caller transitively reaches an ordered region, but whether it violates ordering depends on what the caller does. Human judgment per finding; never auto-PR.

### `lock_held_during_external_call`

- The external call is the lock's purpose: a connection pool's `Get()` holds the pool mutex through the handshake; releasing it defeats the pool.
- Lazy-init under mutex: lock held through the first call's IO so concurrent callers see the populated cache. Alternatives (release-during-IO, sync.Once, singleflight) either thundering-herd duplicate IO or wait identically — the wait is the design. Don't auto-PR, but read the function for asides.
- Distributed locks (`Lock(ctx) error` / `Unlock(ctx)` — Redis mutex, postgres advisory): held through the protected IO by design. A detector that doesn't distinguish them from `sync.Mutex` flags every callsite.
- `time.Now()` tagged `io_sync`: technically a syscall, sub-microsecond; holding a lock through it is fine.

## Volume Reduction

When tightening a caller/sink predicate:

- Down-rank callers already in the sink's EffectClass (e.g. both `external_mutation`); cross-fn detectors are for cross-boundary entries, not intra-region calls.
- Skip generated and vendored code categorically for any cross-fn detector with > 1000 candidates.
- Never tighten against small samples (the ~20 threshold above).

## Reporting

Per detector batch: total candidates, false positives, real-but-low-impact (with reason), real bugs (with PR links). Real bugs / total = the detector's effective precision on this codebase; if it stays below ~5%, tune before running it per-build.

## Related Skills

`golang-code-review` for the fix code before submission; `git-pr` for PR creation.
