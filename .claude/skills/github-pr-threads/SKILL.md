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

Resolve every thread whose underlying issue is handled; leave open only
what's genuinely outstanding. "Addressed" = a pushed commit does exactly
what the thread asked, or a reply explains why current code already
answers it. Pending/deferred/follow-up work is NOT addressed — stays open.

## Rules

- Reply only after the addressing commit is on the remote branch.
- Reply format: "addressed in {short_sha} — {description}". Describe
  what changed, not why the reviewer was right.
- Only resolve threads addressed by commits on THIS PR's branch.
- If a thread asks for X and the commit does Y (related but not X),
  don't resolve.
- Batch: reply to all addressed threads in one pass, then resolve in
  one pass. Always pair reply with resolve, after every round of fixes.

## Voice

Lowercase, no emoji, one sentence (two max), humble, name the specific
change. Never name the reviewer — say "the review" or "feedback".

- "addressed in ac7636e2 — all reserved fields removed, fields renumbered sequentially."
- "good catch, missed this one. added min_len:1, max_len:131072 in 651c4af3."

## Workflow

1. Gather thread state:

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

2. Categorize each open thread: **ADDRESSED** (pushed commit fixes
   exactly what was asked), **DESIGN** (needs discussion — reply, don't
   resolve), **PENDING** (future commit — skip for now).

3. Reply (batch; same body for threads addressed by the same commit):

```bash
for cid in <comment_ids>; do
  gh api "repos/OWNER/REPO/pulls/NUM/comments/$cid/replies" \
    --method POST -f body="addressed in <sha> — <description>."
done
```

4. Resolve (batch):

```bash
for tid in <thread_ids>; do
  gh api graphql -f query="mutation {
    resolveReviewThread(input: {threadId: \"$tid\"}) {
      thread { isResolved }
    }
  }"
done
```

5. Report counts: replied + resolved, and remaining by category.

## Design question threads

Reply with the decision and rationale (and where the implementation
lands if it's a future commit), but leave the thread open for the
reviewer to close. Boundary: "should this be X?" and you made it X →
ADDRESSED, resolve. "what's the vision here?" → reply, leave open.

## Approval

Rote replies ("addressed in {sha} — ...") post directly — the commit
speaks for itself. Non-rote replies — design explanations, rationale,
answers to open questions, pushback — must be drafted and shown to the
user (original comment + proposed reply) and approved before posting.

## Post-push feedback check

After any push to a PR branch, wait ~5 minutes for CI and bot reviews,
then check `gh pr checks` + open threads and triage new items. Automate
the wait (ScheduleWakeup or cron) rather than remembering manually.
