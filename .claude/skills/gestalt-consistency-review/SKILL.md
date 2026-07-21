---
name: gestalt-consistency-review
description: "Reviewer persona for gestalt inconsistencies — code that works and is documented correctly, yet whose behavior is the odd one out relative to its sibling APIs or the implicit contract the rest of the system follows. Catches CRUD/shape asymmetry, error-handling divergence among siblings, naming/argument-order/casing drift, default/nullability asymmetry, unit/type drift (seconds vs ms), idempotency asymmetry, and the abandoned-migration signature (half the call sites new pattern, half old). Use when reviewing a family of analogous endpoints/functions/fields, when adding one member to an existing set, when something feels like the odd one out, or when auditing for convention drift after a half-finished migration. Triggers: consistency, inconsistency, least astonishment, odd one out, outlier behavior, convention drift, asymmetric API, abandoned migration, parity, conceptual integrity, sibling APIs, principle of least surprise, symmetry."
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# Gestalt Consistency Review

Audits for behavior that is locally correct but globally surprising: code, comments, and docs all agree and are each right, yet the behavior is an outlier relative to its sibling APIs or the implicit contract the rest of the system follows. When adding one member to an existing set, the new member is the most likely outlier; the existing set defines the convention.

## Scope boundary

This persona ASSUMES code/comments/docs agree. If they disagree, that is a different axis — hand off and move on, do not absorb it:

- code says A, comment says B → `comment-discipline` (process narration) or `pr-review-toolkit:comment-analyzer` (accuracy). Not this skill.
- code says A, comment says A, docs say A, and A is the odd one out → this skill.

Also out of scope: plain bugs (standard review); security-relevant asymmetry (→ `sharp-edges` / security pass); style nits with no behavioral or contract surprise. "Every sibling is `listX` returning `[]` and this one is `getXs` returning `null`" is a finding; "I'd have named it differently" is not.

## Core posture

Ask of every candidate:

1. **Peer set** — name the siblings. Fewer than 2 named peers = style opinion, not a finding. 2 is thin evidence; 5+ gives a real majority.
2. **Dominant convention** — establish by counting, not preference. Internal consistency (within the family) is the bar; matching broader ecosystem convention is a weaker secondary signal.
3. **Accident or deliberate?** — asymmetry is often the right choice. Only accidental fragmentation is a finding.

## Taxonomy

1. **CRUD / shape asymmetry** — `create` returns the full object, `update` returns `{id}`; one `get` returns `null` on miss, the sibling returns `[]` or throws; `create` and `update` accept different field sets for the same resource; one list paginates in `{items, next}`, the sibling returns a bare array. Expectation: symmetric CRUD with only principled differences (`id` read-only, `created_at` server-set). Most common class.
2. **Error-handling divergence** — one sibling returns `(T, error)`, another `(T, bool)`, a third panics; one wraps errors, the sibling swallows and returns zero; one validates input, the sibling trusts the caller; one logs-and-continues, the parallel one aborts. A reader who learned one contract writes a bug against the other.
3. **Naming / argument-order / casing drift** — `getUserById` / `fetchOrg` / `lookupTeam`; `(ctx, id, opts)` vs `(id, ctx, opts)` (silent footgun when types coincide); `user_id` / `userId` / `UserID` for one concept; `enabled` here, `disabled` in the parallel place.
4. **Default / nullability asymmetry** — a flag defaults `true` here, `false` in the parallel constructor; omitted field means "server default" here, "clear the value" there; empty string as unset vs literal empty.
5. **Unit / type drift** — seconds vs milliseconds across parallel timeouts; epoch `int64` vs `time.Time`; bytes vs KiB; dollars vs cents; string UUID vs bare int64 ID. Highest-severity class: invisible at the type level, produces silent 1000x errors. If siblings must differ, name the unit (`timeoutMs`).
6. **Idempotency / side-effect asymmetry** — one `DELETE` no-ops on an absent resource, the sibling 404s on the second call; one create is upsert-shaped (retry-safe), the analogous one errors on duplicate. A retry wrapper written against the idempotent sibling silently corrupts state at the other.
7. **Abandoned-migration signature** — half the call sites use the new pattern, half the old, each half internally consistent and correctly documented (`doThing` and `doThingV2` both live, both called). The finding is not "the old pattern is wrong" but "two contracts for one job; which one you get depends on where you landed." The next author extends whichever half they land in.

## Method

1. Enumerate the peer set (handlers under a route group, `getX` methods on a type, config fields, both halves of a suspected migration). Write it down.
2. Tabulate each member per taxonomy axis; the plurality is the convention.
3. Flag outliers with the specific axis and deviation ("returns `null` where the other four return `[]`"), not "feels inconsistent."
4. Hypothesize the historical cause and test cheaply with `git log`/`git blame` — the introducing commit often names it:
   - historical quirk (written before the convention existed)
   - since-removed limitation (workaround whose constraint is gone)
   - abandoned remediation (was supposed to be swept in, wasn't)

   A deviation with a *live* cause is not a finding.

## Verification — two gates before any finding ships

- **Gate 1**: state in one sentence the expectation a sibling-reader would carry and why the domain makes it reasonable. "They should match because matching is nice" fails the gate.
- **Gate 2**: search for a deliberate reason — a comment/ADR explaining the divergence, or a domain/performance/security/correctness constraint forcing it. Canonical trap: an endpoint returning **404 rather than 403** on an unauthorized resource *on purpose*, to avoid leaking existence. Its 403 siblings are not an inconsistency; "fixing" it breaks a security control.

Both gates must pass. If a deliberate reason exists, the verdict is **justified exception, not a finding** — the useful output is "add a one-line note at the divergence so the next reviewer doesn't re-flag it," not "make it match."

## Severity

| Severity | Criteria |
|---|---|
| High | Silent wrong behavior for a convention-trusting reader: unit/type drift, idempotency asymmetry under a shared retry path, argument-order swap with coinciding types. |
| Medium | Surprising shape/contract forcing per-member special-casing: CRUD shape asymmetry, error-idiom divergence, null-vs-empty on a read. |
| Low | Cognitive-load drift without behavior change: naming/casing/verb drift, default flips on rarely-set flags. |
| Not a finding | Justified exception, <2 named peers, or pure style preference. |

Score an abandoned migration by its worst divergence, plus one note that the split itself (two contracts for one job) is the root the findings share.

## Output

Per finding:

- **Peer set** — named members.
- **Convention** — dominant shape/idiom/unit, with count ("4 of 5 return `[]`").
- **Outlier** — `path/to/file:line`, exact deviation.
- **Axis** — CRUD-shape / error-handling / naming / default-nullability / unit-type / idempotency / abandoned-migration.
- **Reader's wrong expectation** — the one-sentence surprise (Gate 1).
- **Deliberate-reason check** — what was searched and not found (Gate 2), or why this is a justified exception.
- **Hypothesized cause** — quirk / removed-limitation / abandoned-remediation, with `git blame` evidence if cheap.
- **Severity** — per the table.
- **Suggested direction** — align the outlier to the convention, OR align the siblings to the outlier (see Common Mistakes #5), OR add a one-line note documenting the deliberate divergence.

End with `N findings · M justified exceptions · K abandoned-migration roots`. If the set is consistent, say `no findings` and name the set checked. Report only; do not rewrite the files.

## Common Mistakes

1. **Flagging a justified exception.** The 404-not-403 case is the archetype — "fixing" it can break a real security control. If you skipped Gate 2, you don't have a finding yet.
2. **Treating intentional variation as drift.** `parseConfig` returning a struct and `parseArgs` returning a slice are not inconsistent — analogy in name is not analogy in domain. Gate 1 is the filter.
3. **Mistaking personal preference for inconsistency.** If the whole set says `get`, the set is consistent; your preference for `fetch` is out of scope. The convention is what the set does, established by counting.
4. **Demanding symmetry where domains genuinely differ.** Two `delete`s may operate on resources with genuinely different existence semantics. Manufacturing symmetry against the grain of the domain is worse than the drift.
5. **"Fixing" the outlier when the outlier is the correct one.** The most expensive mistake. Counting establishes the convention, not the correct behavior — sometimes the lone member is the fixed one and the siblings are the stragglers. Then the finding is "align the N siblings to the outlier," which is larger and more valuable. An abandoned migration has a right half and a left-behind half; do not flatten toward the stale majority.
