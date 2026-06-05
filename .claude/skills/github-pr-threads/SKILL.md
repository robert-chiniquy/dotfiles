---
name: github-pr-threads
description: >-
  Reply to and resolve GitHub PR review threads after pushing fixes.
  Use after committing changes that address PR feedback — matches
  threads to commits, posts replies with commit references, and
  resolves threads. Triggers on: resolve threads, reply to PR
  feedback, address PR comments, close review threads.
---

# GitHub PR Thread Management

After pushing fixes that address PR review feedback, use this pattern
to reply and resolve the corresponding threads.

## Why resolve?

Unresolved threads are the reviewer's TODO list for a skim. If
threads that have been sufficiently addressed stay open, the reviewer
has to re-read every one to find what's actually outstanding. Keep
the PR in the state a human should scan: **resolve every thread
whose underlying issue is handled; leave open only truly outstanding
or unaddressed issues**. That way the open-thread count is itself
a reliable signal.

A thread is "sufficiently addressed" when the pushed commit does
exactly what the thread asked for, or when a reply has explained
why the current code answers the question. Pending, deferred, or
follow-up work is not "addressed" — those stay open (see Design
Question Threads below).

## Rules

1. **Only reply after the fix is pushed.** Never reply to a thread
   before the commit that addresses it is on the remote branch.

2. **Always reference the commit.** Format: "addressed in {short_sha}
   — {description}". The description should be succinct and humble.
   Describe what changed, not why the reviewer was right.

3. **Only resolve threads you actually addressed.** If a thread asks
   for X and the commit does Y (related but not X), don't resolve it.
   If the thread is a design question that needs discussion rather
   than a code change, don't resolve it — reply with the answer but
   leave it open for the reviewer to close.

4. **Batch operations.** Reply to all addressed threads in one pass,
   then resolve them in one pass. Don't interleave with other work.

5. **After every round of fixes, resolve.** Replying without
   resolving is worse than not replying — it grows the thread but
   does not reduce the reviewer's workload. Pair the reply and the
   resolve.

## Voice

- Lowercase, no emoji
- Brief — one sentence, two max
- Humble — describe the change, don't explain why it's better
- Name the specific improvement when useful (e.g., "converted to
  LatchkeyVaultVariant enum" not just "fixed")

Examples:
- "addressed in ac7636e2 — all reserved fields removed, fields renumbered sequentially."
- "addressed in 08f95ba1 — renamed across all latchkey protos."
- "addressed in 614eebe0 — added Batch as a quantity modifier in extractSemanticPrefix, plus 11 Latchkey RPC overrides in finalOverrides."
- "good catch, missed this one. added min_len:1, max_len:131072 in 651c4af3."

Never attribute comments, ideas, or feedback to specific people by
name in replies or any output. Say "the review" or "feedback" rather
than naming the reviewer.

## Workflow

### 1. Gather thread state

```bash
gh api graphql -f query='query {
  repository(owner: "OWNER", name: "REPO") {
    pullRequest(number: NUM) {
      reviewThreads(first: 100) {
        nodes {
          id
          isResolved
          comments(first: 1) {
            nodes { databaseId author { login } body path line }
          }
        }
      }
    }
  }
}'
```

### 2. Categorize threads

Map each open thread to one of:
- **ADDRESSED** — a pushed commit fixes exactly what was asked
- **DESIGN** — needs discussion, not a code change (reply but don't resolve)
- **PENDING** — will be addressed in a future commit (skip for now)

### 3. Reply (batch)

```bash
for cid in <comment_ids>; do
  gh api "repos/OWNER/REPO/pulls/NUM/comments/$cid/replies" \
    --method POST -f body="addressed in <sha> — <description>."
done
```

Use the same reply body for threads addressed by the same commit.
Different commits get different reply bodies.

### 4. Resolve (batch)

```bash
for tid in <thread_ids>; do
  gh api graphql -f query="mutation {
    resolveReviewThread(input: {threadId: \"$tid\"}) {
      thread { isResolved }
    }
  }"
done
```

### 5. Report

After resolving, report the counts:
- Threads replied + resolved
- Threads remaining (with categories)

## Design Question Threads

When a reviewer asks a design question (not requesting a specific
code change), reply with the decision and rationale but leave the
thread open. The reviewer should close it once satisfied.

Format: answer the question directly, cite the reasoning, and note
where the implementation will land if it's a future commit.

Example: "vault_id will be server-assigned (KSUID via uid.New()) —
the MLS runtime receives it as input, doesn't generate it. removing
from LatchkeyVaultInput in the Phase 1 proto alignment work."

## Approval Workflow

**Rote replies** (e.g., "addressed in {sha} — removed reserved fields")
can be posted directly. The commit speaks for itself.

**Non-rote replies** — design explanations, rationale for decisions,
answers to open questions, pushback on suggestions — must be drafted
and shown to the user before posting. Present:

1. Paul's original comment (verbatim or summarized)
2. Your proposed reply
3. Ask for approval or edits

Only post after the user approves. This applies to any reply that
requires judgment rather than just referencing a commit.

## Post-Push Feedback Check

After any push to a PR branch, wait ~5 minutes for CI and bot reviews
to run, then check the PR for new feedback. This catches bot-generated
suggestions, CI failures, and reviewer comments before moving on to
the next task.

Pattern: push → wait 5 min → check `gh pr checks` + open threads →
triage new items → either address or note for later.

Use ScheduleWakeup or a cron job to automate this rather than
manually remembering.

## Common Mistakes

- **Resolving before pushing** — the reviewer can't verify the fix
  if the commit isn't on the branch yet.
- **Resolving design threads** — if the reviewer asked "should this
  be X?" and you made it X, resolve. If they asked "what's the
  vision here?" reply with the answer but let them close it.
- **Generic replies** — "fixed" or "done" with no commit reference.
  Always include the sha and what changed.
- **Resolving threads addressed by a different PR** — only resolve
  threads with commits on THIS PR's branch.
