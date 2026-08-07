---
name: github-pr-threads
description: >-
  Reply to and resolve GitHub PR review threads after pushing fixes.
  Use after committing changes that address PR feedback — matches
  threads to commits, posts replies with commit references, and
  resolves threads. Triggers on: resolve threads, reply to PR
  feedback, address PR comments, close review threads. Also use
  proactively after a fix push when open review threads remain.
---

# GitHub PR Thread Management

Resolve every thread whose underlying issue is handled; leave open only
what's genuinely outstanding. "Addressed" = a pushed commit on **this PR's
branch** does exactly what the thread asked, or the current code already
answers it. Pending/deferred/follow-up work is NOT addressed — stays open.

## Common Mistakes

1. **Leaving fixed threads open** — after a fix push, MUST `Addressed in <sha>` + resolve without waiting to be asked.
2. **Resolving without a pushed commit on this PR's branch** — never resolve on local-only or wrong-branch SHAs.
3. **Resolving "related" work** — commit does Y while thread asked X → leave open.
4. **Skipping the reply** — resolve alone is incomplete; reply format is required.
5. **Design questions resolved as fixed** — leave open unless the decision is already implemented.

**Mandate (also in `~/.claude/Claude.md`):** for every addressed thread, post
`Addressed in <sha>` (optional short context), then mark resolved. No approval
needed for that reply. **MUST NOT** leave fixed threads open.

## Rules

- Reply only after the addressing commit is on the remote branch.
- **Reply format (required):** `Addressed in <sha>` — optional few words naming
  the change. Prefer abbreviated or full SHA; optionally link the commit URL.
  Example: `Addressed in ac7636e2 — removed reserved fields; renumbered sequentially.`
- Only resolve threads addressed by commits on THIS PR's branch.
- If a thread asks for X and the commit does Y (related but not X), don't resolve.
- Batch: reply to all addressed threads, then resolve them. Always pair reply
  with resolve after every round of fixes.
- Stale bot threads whose ask is already fixed: still post `Addressed in <sha>`
  (or note the fixing commit range) and resolve — same as human threads.

## Voice

No emoji, one sentence (two max), humble, name the specific change. Never name
the reviewer — say "the review" or "feedback". Prefer the `Addressed in <sha>`
prefix so replies stay consistent with global config.

- `Addressed in ac7636e2 — all reserved fields removed, fields renumbered sequentially.`
- `Addressed in 651c4af3 — added min_len:1, max_len:131072.`

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
          isOutdated
          comments(first: 1) {
            nodes { databaseId author { login } body path line }
          }
        }
      }
    }
  }
}'
```

2. Categorize each open thread: **ADDRESSED** (pushed commit fixes exactly
   what was asked), **DESIGN** (needs discussion — reply if approved, don't
   resolve), **PENDING** (future commit — skip for now).

3. Reply (batch; same body for threads fixed by the same commit):

```bash
# Prefer in_reply_to on the first comment of the thread:
gh api "repos/OWNER/REPO/pulls/NUM/comments" \
  -f body="Addressed in <sha> — <description>." \
  -F in_reply_to=<comment_database_id>
```

4. Resolve (batch):

```bash
gh api graphql -f query='mutation($id:ID!) {
  resolveReviewThread(input: {threadId: $id}) {
    thread { isResolved }
  }
}' -f id="<thread_node_id>"
```

5. Report counts: replied + resolved, and remaining by category.

## Design question threads

Reply with the decision and rationale (and where the implementation lands if
it's a future commit), but leave the thread open for the reviewer to close —
unless the decision is already implemented, then treat as ADDRESSED and use
`Addressed in <sha>`. Boundary: "should this be X?" and you made it X →
ADDRESSED, resolve. "what's the vision here?" → reply (approval required if
not rote), leave open.

## Approval

- **No approval:** `Addressed in <sha>` replies on genuinely fixed threads, and
  resolve/unresolve alone.
- **Needs approval:** design explanations, rationale beyond a short change
  note, answers to open questions, pushback, or any non-rote PR comment.

## Post-push feedback check

After any push that is meant to address review feedback:

1. Confirm the fix commit is on the remote.
2. Match open threads to that commit (or earlier commits on the branch).
3. For each ADDRESSED thread: `Addressed in <sha>` + resolve.
4. Optionally wait for bot re-review, then triage any new threads the same way.

Do not wait for the user to ask "resolve threads" when the fix is already
pushed and the match is clear.

## Before finishing

- [ ] Fix commit on remote this PR branch?
- [ ] Every ADDRESSED thread: `Addressed in <sha>` + resolved?
- [ ] No false resolves (Y≠X)?
- [ ] Design threads left open unless implemented?
- [ ] Counts reported?
