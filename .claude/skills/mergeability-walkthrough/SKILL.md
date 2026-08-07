---
name: mergeability-walkthrough
description: >-
  Walk open PRs one at a time with multiple-choice decisions and explicit risk
  framing (API backcompat, wire/proto compat, client on-disk state, stack
  ancestry). Use when the user asks to walk mergeability, greenlight merges one
  by one, "merge talk" / "PR walk", or "mergeability process" / "pass over the
  set".
---

# Mergeability walkthrough

Default: one PR at a time; do not merge until the user authorizes **that** PR.
After a full-pass recap, the user may authorize a **batch** of named actions
(e.g. merge base + restack children + undraft) in one reply — execute only
those, in stack order.

Complements `pr-pass` (cohort triage / ready list). This skill is the
decision walk after a set is in hand.


## Common Mistakes

- Batch greenlights without cards (or without a confirmed multi-action recap).
- "Merge now" with red CI, open **inline** threads, or unanswered human review.
- Treating bot `CHANGES_REQUESTED` (0 threads) as open human review threads.
- Merging a child before base is on trunk / without ancestor check.
- Using GraphQL merge on a GitHub stack without `merge-async`.
- Rebase of 50+ commits when merge-trunk-into-tip would do.
- API/wire compat from `origin/<trunk>` without deploy SHA (or explicit unchecked).
- Shipping an experimental path as the product default without saying so.
- Ignoring on-disk/key state when a server-only PR changes next-session client writes.
- Treating external SDK / library PRs as product delivery by themselves.
- Enqueueing a merge queue without an option that explicitly authorizes it.
- Claiming green CI on a pre-restack SHA after force-push.

## When to use

- Walk PRs, decide mergeability, greenlight merges.
- After a mergeability list: convert into this walk (highest-value ready first;
  respect stack bottoms before children).

## Protocol

0. **Re-check status immediately before EACH card** — not once at walk start.
   Fetch `state`, `isDraft`, `mergeStateStatus`/`mergeable`, failing/pending
   checks, unresolved thread count, base/head SHAs. If the world moved since
   the last card, say so and adapt.

1. **Order** — Prefer: product-critical first; docs/low-risk next; shared
   libraries carefully; **stacks bottom-up**; experimental / external-SDK last
   or skip. Project skills may override order for domain stacks.

2. **UI**
   - Default: one multiple-choice card (`ask_user_question`) per PR; wait for
     the answer before the next card. Pair with `questioning-the-user`.
   - If the user declines the tool, says "pass again", "just report", or "status
     then decide": render **all cards in one prose pass** (same axes/gates),
     then a single multi-action line they can confirm (merge X; restack Y;
     undraft Z). Do not stall.

3. **Hard process gates** (report every card; block bare "merge now" unless true)
   - **CI green** — required checks SUCCESS (or explicit non-blocking/skipped).
     Re-fetch; no stale snapshots.
   - **No unresolved inline review threads** — list them. "Merge now" not offered
     if any remain (Hold / fix / explicit user waive).
   - **Feedback taxonomy** (do not conflate):

     | Kind | "Merge now"? |
     |------|----------------|
     | Unresolved **inline** threads | Hard block until fixed or user-waived |
     | Bot `CHANGES_REQUESTED` (CI judge / policy bot), **0** open threads | Offer **merge with policy override** as explicit option; do not treat as open human threads |
     | Org approval check only (required review/team gate, not code feedback) | Process gate; not a code-fix task |
     | Human "request changes" review | Hold or fix; no silent LGTM |

   - **Stack ancestor** — for a child PR, `merge-base --is-ancestor <base-branch-tip> <child-tip>` must hold (or base is already trunk after parent merged). Else offer **restack**, not merge.
   - Draft: undraft is a separate option; do not merge drafts.

4. **Each card must include**
   - What it does (one sentence)
   - State — draft, reviewDecision, CI summary, conflicts, **unresolved thread
     count**, feedback kind (table above)
   - Shared surface — none / monorepo-only / published SDK or library pin
   - **Prod API backcompat** when the PR calls or requires server behavior —
     vs **deployed** production SHA when known (below), not only `origin/main`
   - Wire/proto compat when the PR touches schemas or generated stubs
   - Client on-disk / durable local state when relevant
   - Secret or device key material when relevant (unlock of older clients?)
   - Stack position — base PR, children, restack needed?
   - Recommendation — first option `(Recommended)` with why

5. **Options** (reshape per PR)
   - Merge now / merge-async / enqueue — only if §3 hard gates pass
   - Merge with **policy override** (bot CHANGES_REQUESTED, threads clear)
   - Restack onto base tip / onto trunk after base merge
   - Hold for CI / threads / undraft / fix X
   - Skip / later / close  
   Never invent a fake "Other" (the tool adds it).

6. **After each answer** — Act only what was authorized. Do not merge the next
   PR unprompted. Then next card (or stop if batch already fully executed).

7. **After the walk** — Compact recap; no re-confirm.

## Stacks (GitHub stacked PRs)

- Detect: children with `base` = parent's head branch; stack API / "part of a stack".
- **Land bottom-up.** Do not merge a child until its base is on the target trunk
  (or child is retargeted and green against that trunk).
- After base merges: restack remaining children onto `origin/<trunk>` (or new
  base tip); re-CI; re-check ancestor gate.
- **Merge API:** GraphQL/`gh pr merge` often fails with *"part of a stack… use
  the asynchronous merge REST API"*. Use:

  ```bash
  # Merge this PR (and, for stack merges, layers below it up to this PR — verify UI/docs)
  SHA=$(gh api repos/OWNER/REPO/pulls/N --jq .head.sha)
  gh api -X PUT "repos/OWNER/REPO/pulls/N/merge-async" \
    -f merge_method=merge \
    -f merge_action=direct_merge \
    -f sha="$SHA"
  # Poll: GET repos/OWNER/REPO/pulls/N/merge-async/{uuid}
  ```

  Prefer `direct_merge` only when user authorized merge and branch rules allow
  (admin/`--admin` equivalent if required for policy override). If async merge
  would pull unwanted upper layers, dissolve/retarget stack first or merge only
  the bottom layer after children no longer block.

## Updating a branch onto trunk

| Situation | Prefer |
|-----------|--------|
| Small stack (few commits) | `git rebase origin/<trunk>` |
| Large base (dozens of commits) or rebase conflicts on early commits | `git merge origin/<trunk>` into tip; regenerate language lockfiles with the project's tool (e.g. `cargo generate-lockfile`, `go mod tidy`), not hand-union of lockfiles |
| Child after parent merged | Rebase child onto `origin/<trunk>` |

Corrupt lock symptom: audit/tooling panics on a missing package version ref —
regenerate the lockfile; do not paper over with text edits.

## Post-restack CI triage

Bucket failures before "fix product code" or "block forever":

| Bucket | Examples | Default move |
|--------|----------|--------------|
| **Compile / lock / audit** | missing symbol, bad lockfile, security-audit tree panic | Fix on the PR |
| **PR-surface tests** | fails in code the PR owns | Fix on the PR |
| **UI / e2e flake** | timeout on UI the PR did not touch; trunk green | Harden test if cheap; else re-run; note flake on card |
| **Infra / tool** | boot smoke timeout, install-action flake | Re-run; no product rewrite without evidence |

Never claim CI green from a run against a pre-restack SHA.

## Deployed production SHA

When the PR **calls or requires** server RPC/behavior that depends on what is
live (not only what is on trunk):

1. Resolve the currently **deployed** server git SHA from the project's ops
   source of truth (deploy bot PR, release tag, cluster metadata — project
   skill or docs name it).
2. Check that SHA's protos/API surface — not `origin/<trunk>` alone.
3. Card verdict: **compatible with deployed `SHA`** / **needs newer server than
   deployed** / **breaking for older clients** / **unchecked — state unknown**
   (only if deploy lookup blocked; say so explicitly).

Do not silently skip when the PR has a live-server dependency.

## Wire / proto compat check

Run when the PR touches `.proto`, generated stubs, vendored wire packages, or
client code that decodes new fields/RPCs.

| Check | How |
|-------|-----|
| **Deployed baseline** | Same deploy → server git SHA as above |
| **RPC add/remove/rename** | New server RPC = additive for old clients; client requiring it needs server **deployed**. Remove/rename = break |
| **Field number / type / oneof** | Additive optional OK; reuse/type change/required = breaking |
| **Enum values** | New values: fail-closed or ignore. Remove/renumber = break |
| **Vendored / pinned wire vs server** | Still a superset of what deployed server serves? |
| **Generated stubs** | Match generator output; no hand-edited wire structs vs source schema |
| **Verdict** | One of the four outcomes above, with SHA or **unchecked** |

## Risk axes (always fill)

| Axis | Ask |
|------|-----|
| **Shared library / SDK** | Monorepo-only vs published pin? One pin across clients? External package repo? |
| **Prod API** | New RPC? Required vs optional? vs **deployed** SHA? |
| **Wire / proto** | Additive vs breaking; deployed verdict |
| **Client on-disk** | State dir, journals, migrations, identity blobs? |
| **Key / device material** | Key roots, enclave/TPM, long-lived tokens? Old devices unlock? |
| **Process** | Draft? CI? Threads? Feedback **kind**? Stack ancestor? Queue? No internal tracker IDs in published text |

Project skills may add domain axes; keep these generic axes filled even then.

## Before finishing

- [ ] Status re-fetched for the PR about to act?
- [ ] Hard gates (CI/threads/ancestor) reported honestly?
- [ ] Acted only on user-authorized PRs?
- [ ] Stack order bottom-up respected?
