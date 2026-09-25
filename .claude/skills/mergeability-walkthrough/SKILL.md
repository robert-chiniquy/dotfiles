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

Default: one PR at a time, one decision per card — but **collect** the
decisions and execute the accumulated actions at the **end of the pass**, in
stack/dependency order. Every card's options include **"Act now on what
you've got"**, which executes everything authorized so far immediately and
then continues the walk. Do not act between cards otherwise. After a
full-pass recap, the user may also authorize a **batch** of named actions in
one reply — execute only those.

Complements `pr-pass` (cohort triage / ready list). This skill is the
decision walk after a set is in hand.


## Common Mistakes

- Walking in subject-matter order, or opening on the most interesting PR. The
  one needing a decision you cannot resolve stalls the walk with the easy
  merges still unasked behind it.
- Acting between cards without an explicit "Act now on what you've got"
  (collect-then-execute is the default).
- Presenting a stale observation or an assumption as a verified current fact.
- Asking the human about work that was optionality-safe to just do first
  (rebase, thread replies, information gathering).
- Mentioning a non-issue (stacks when the PR is not in a stack, proto when
  the diff does not touch wire, keys when no key material, a merge queue
  when the repo has none, "no risk: …" rollups). Omit it.
- A generic checklist card that does not make *this* PR's change and the
  merge risk of *that* change visible.
- Condensing the substance of this PR's change, or hiding it behind a
  summary — a card may use the whole screen for what is live.
- Self-waiving a failing check via attribution reasoning instead of
  presenting the evidence and letting the human decide.
- Merging a production-code draft, or folding RFR into merge for a code PR. A qualifying pure-tests, pure-docs, and/or pure-lint/lsp PR may be marked ready and merged together under the standing grant.
- Manual re-CI choreography when a merge queue already re-tests entries —
  or treating plain auto-merge as if it were a queue.
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
- Enqueueing a merge queue without an option that explicitly authorizes it. Exception: a qualifying pure-tests and/or pure-docs PR (Claude.md standing grant) is already authorized; merge it, do not card it.
- Asking the human to merge a PR whose remaining diff is only tests/fixtures and/or docs that already belong in git, with CI green on the merge SHA. That is a standing grant. Production comments, CI, config, engine, and stdlib are not docs.
- Claiming green CI on a pre-restack SHA after force-push.
- Merging on review state observed earlier in the walk without the
  last-second sweep for review comments that landed since.
- A card or question that does not contain the PR's full `https://github.com/owner/repo/pull/N` URL as visible copyable text (title-only, `#N`, or a markdown link whose anchor hides the URL).
- Treating a rejected `(Recommended)` choice as noise. A non-recommended pick is a data point: note it, hypothesize why, adapt later cards. Do not re-argue the old recommendation.

## When to use

- Walk PRs, decide mergeability, greenlight merges.
- After a mergeability list: convert into this walk, sorted simplest
  resolution first (step 1); stack bottoms still precede their children.

## Protocol

0. **Re-check status immediately before EACH card** — not once at walk start.
   Fetch `state`, `isDraft`, `mergeStateStatus`/`mergeable`, failing/pending
   checks, unresolved thread count, base/head SHAs. If the world moved since
   the last card, say so and adapt.

0.5 **Standing grants first** - if any authorization already covers a card,
   name it. Standing grant (Claude.md, 2026-09-01): a PR whose remaining
   diff is only tests/fixtures, documentation that already belongs in git,
   pure lint/lsp, or a combination of those, may be marked ready and merged
   without asking when CI is green on the SHA that will merge (post-rebase
   if the base moved). A fully covered card is inform-only, then merge it.
   Partially covered: ask only the uncovered part. Omit this line when no
   grant applies.

0.7 **Bring each PR to readiness before its card.** Rebase only PRs the
   user opened, unless the user names another author's PR. The only reason
   NOT to have already done a thing (rebase onto the moved base of an
   allowed PR, reply to and resolve addressed threads, fix what unresolved
   threads ask, run any purely information-gathering action) is a GOOD
   reason that requires the human. Everything optionality-safe that improves
   the readiness or the quality of the human's decision is done first, so
   the card presents a PR at maximal readiness, not a to-do list. Decision
   actions (merge, enqueue, close, RFR) still wait for authorization; this
   step is everything before those.

1. **Order: simplest resolution first.** Sort by the work standing between the
   PR and merge, not by subject matter: green with no open threads → needs a
   rebase or a re-run → has threads to address → needs a decision from the
   human → blocked on another PR or repo. Sort AFTER step 0.7, since bringing
   a PR to readiness can move it up a tier.

   A walk of N PRs is N turns of the human's attention. Opening with the PR
   most likely to stall spends it on the one decision that cannot be closed in
   that turn, and the trivial merges never get asked. Landing the easy ones
   first also changes what is left, since each merge re-bases the rest.

   Tiebreak within a tier: product-critical first; docs/low-risk next; shared
   libraries carefully; experimental / external-SDK last or skip. Stacks
   override the sort: bottom-up regardless of which tier each PR is in.
   Project skills may override the tiebreak for domain stacks.

2. **UI**
   - Default: one multiple-choice card (`ask_user_question`) per PR; wait for
     the answer before the next card. Pair with `questioning-the-user`.
   - **URL on every card.** The question text (and any prose-pass card) MUST
     include the PR's full URL as literal visible text:
     `https://github.com/owner/repo/pull/N`. First line of the question.
     Not `#N` alone, not a title-only header, not a markdown link that hides
     the URL. Same rule if one card names more than one PR: every PR gets
     its own bare URL.
   - **This PR, not a template.** Lead with what this diff does and what
     merging *this* change can break. Hard gates and risk axes are a private
     checklist; print an axis only when it is live for this PR. Do not name
     stacks, proto, keys, queues, or grants that do not apply.
   - **Never condense a live issue.** Full detail inline for the change and
     its merge risks — a card may use the entire screen for that. Empty
     categories are not substance; omit them.
   - If the user declines the tool, says "pass again", "just report", or "status
     then decide": render **all cards in one prose pass** (same axes/gates),
     then a single multi-action line they can confirm (merge X; restack Y;
     undraft Z). Do not stall. Each prose card still starts with the bare URL.

3. **Hard process gates** (report every card; block bare "merge now" unless true)
   - **CI green, attributed** — required checks SUCCESS on the current SHA.
     Re-fetch; no stale snapshots; read the **rollup** (per-check surfaces
     disagree with it). A failing check is presented WITH attribution
     evidence, collected for the human: does trunk fail the same check
     identically? Is it a known advisory or environment-divergent gate
     (e.g. a deadline-bound test that splits local/CI)? The human decides
     whether an attributed failure blocks — never self-waive one.
   - **No unresolved inline review threads** — list them. "Merge now" not offered
     if any remain (Hold / fix / explicit user waive).
   - **Feedback taxonomy** (do not conflate):

     | Kind | "Merge now"? |
     |------|----------------|
     | Unresolved **inline** threads | Hard block until fixed or user-waived |
     | Bot `CHANGES_REQUESTED` (CI judge / policy bot), **0** open threads | Offer **merge with policy override** as explicit option; do not treat as open human threads |
     | Org approval check only (required review/team gate, not code feedback) | Process gate; not a code-fix task |
     | Human "request changes" review | Hold or fix; no silent LGTM |

   - **Stack ancestor** — only when this PR is a stack child:
     `merge-base --is-ancestor <base-branch-tip> <child-tip>` must hold
     (or base is already trunk after parent merged). Else offer **restack**,
     not merge. Do not mention stacks on an independent PR.
   - **Draft lifecycle** — a draft stays draft until the user approves
     ready-for-review, except a qualifying pure-tests and/or pure-docs PR
     (standing grant) which may be marked ready and merged without asking.
     A ready PR stays ready until merged. RFR approval is its own card
     decision except under that grant.

4. **Each card must include**
   - **The PR URL** — first line, bare `https://github.com/owner/repo/pull/N`
   - **What this PR changes** — specific enough that the human can see the
     merge blast radius (which API, file family, or invariant moves). Not a
     restatement of the title alone.
   - **Merge risk of that change** — only the live ones (examples: deployed
     API vs this RPC, proto field reuse, on-disk format, key material, a
     stack child whose parent is not on trunk). Check the axes below
     privately; print an axis only when it applies to this diff.
   - **State that constrains the decision** — draft, CI on *this* SHA,
     unresolved inline threads, feedback kind if it blocks or changes the
     options. Skip fields that do not change the options.
   - **Epistemic status on every claim** — verified this fetch, old
     observation (say when), or assumption. Never let one read as another.
   - Recommendation — first option `(Recommended)` with why, tied to this
     PR's change and gates

5. **Options** (reshape per PR; omit ones that cannot apply)
   - Merge now / merge-async / enqueue — only if §3 hard gates pass
   - Merge with **policy override** — only if bot CHANGES_REQUESTED and
     threads are clear
   - Restack — only if this PR is behind its base or is a stack child
   - Hold for CI / threads / undraft / fix X — only if that is the live block
   - Skip / later / close
   Never invent a fake "Other" (the tool adds it).

6. **After each answer** — record the decision and show the next card. Act
   only at the end of the pass, on an "Act now on what you've got", or on a
   directly named action — and never beyond what was authorized.
   If the pick was not `(Recommended)`, treat that as evidence: write a
   short note (what they chose vs what you offered, and a hypothesis —
   wrong gate, they want RFR not merge, different workstream, they
   waived a risk you treated as blocking). Use it on later cards. Do not
   re-ask or defend the unused recommendation.

7. **Executing accumulated merges (no queue)** — sequentially: update each
   PR onto the moved trunk, wait for full CI green against that exact base,
   then merge, then the next. N individually-green PRs do not imply their
   combination is green. A merge queue replaces this choreography (below).
   Decisions were collected earlier in the pass, so **re-fetch each PR's
   state again immediately before acting on it** — if the world moved since
   its card (new commits, new threads, changed checks), surface that instead
   of executing the stale decision.

8. **After the walk** — Compact recap; no re-confirm.

## Last-second review sweep (before every merge action)

Immediately before ANY merge action — `gh pr merge`, enqueueing into a
merge queue, or arming auto-merge — re-fetch the PR's reviews, inline
threads, and issue comments one final time and look for anything that
requests changes. Review triage done earlier in the walk (or in a prior
turn) is stale by definition: a human or bot review can land between the
card and the click. A `CHANGES_REQUESTED` review, a new inline thread, or
a comment asking for changes found in this sweep pauses that PR's merge
and surfaces the content to the human; bot nits already triaged as
non-material do not re-block, but a NEW finding does until dispositioned.
One `gh pr view <n> --json reviews,reviewRequests` plus a comments fetch
`--since` the last look is enough. This applies per-PR in a batch: each
entry gets its own sweep at its own enqueue moment, not one sweep for
the batch.

## Merge queues

When the target branch has a merge queue: **enqueueing is the merge action**
(offer it as such), and the queue's own rebase-and-retest of each entry
against the live target satisfies the re-CI-between-merges rule — so arming
several entries from one authorized batch is sound where plain auto-merge is
not. `gh pr merge --auto` may report "merge strategy is set by the merge
queue"; a plain `gh pr merge` enqueues. Watch entries that LEAVE the queue
(`mergeStateStatus` DIRTY/BLOCKED after enqueue) — that is a walk event to
surface, not a silent state.

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

## Risk axes (check privately; print only if live)

| Axis | Ask (print only when this PR hits it) |
|------|-----|
| **Shared library / SDK** | Monorepo-only vs published pin? One pin across clients? |
| **Prod API** | New RPC? Required vs optional? vs **deployed** SHA? |
| **Wire / proto** | Additive vs breaking; deployed verdict |
| **Client on-disk** | State dir, journals, migrations, identity blobs? |
| **Key / device material** | Key roots, enclave/TPM, long-lived tokens? Old devices unlock? |
| **Process** | Only the gates that change options: draft, red/pending CI, open threads, blocking feedback, stack child, queue |

Do not print "no risk" / "N/A" / "not a stack" / "no proto change". Absence is
silence. Project skills may add domain axes under the same omit-if-quiet rule.

## Before finishing

- [ ] Status re-fetched for the PR about to act?
- [ ] Hard gates that actually apply reported honestly?
- [ ] Every card starts with the PR's full copyable GitHub URL?
- [ ] Card is about this PR's change and live merge risks; non-issues omitted?
- [ ] Acted only on user-authorized PRs?
