---
name: pr-pass
description: Triage open GitHub pull requests for actionable feedback, unresolved review threads, requested changes, CI or merge blockers, and ready-to-merge links. Use when the user asks for a PR pass, review/mergeability sweep, status check across their open PRs, a filtered PR cohort, or asks which PRs need action versus can be merged.
---

# PR Pass

## Overview

Run a focused GitHub PR sweep. Report only what needs action and which PRs are ready to merge; avoid long status dumps.

Do not merge, assign reviewers, post comments, or push changes unless the user explicitly asks for that write action.

## Workflow

1. Resolve scope.
   - If the user names repos, orgs, authors, PR URLs, branches, labels, base branches, or a provenance filter, use that scope.
   - If the user says "my PRs", identify the authenticated GitHub user with `gh api user`.
   - If the user names a cohort like "auth PRs", "release PRs", "generated-code PRs", or "go-analysis PRs", infer matching title, branch, body, label, and repo markers; keep the filter conservative and say what was included.
   - Exclude unrelated stacks or cohorts unless the user explicitly includes them.

2. Fetch current PR state.
   - Prefer GitHub structured data through `gh api graphql` or `gh pr view --json`.
   - Fetch at least: title, URL, state, draft status, head branch, body, labels, review decision, merge state, mergeability, latest check state, issue comments, reviews, and review threads.
   - When GraphQL reports `UNKNOWN` mergeability for an otherwise likely-ready PR, refresh with REST:

```bash
gh api repos/OWNER/REPO/pulls/NUMBER --jq '{number, mergeable, mergeable_state, draft}'
```

3. Classify actionable feedback.
   - Action needed: any unresolved thread with reviewer feedback not already answered by the author, any `CHANGES_REQUESTED` review, failed checks, draft state when the PR is expected to be ready, or merge conflicts.
   - Needs decision: unresolved technical question where the author is waiting on someone else, or a thread that changes whether the PR should merge.
   - No action: bot-only comments with no unresolved thread, resolved threads, author self-comments, approvals, and old comments already addressed by later commits or replies.
   - Do not count GitHub Actions review summaries as human review. Mention them only if they contain unresolved suggestions or failures.

4. Classify ready-to-merge PRs.
   - Include only open, non-draft PRs with successful checks, no unresolved threads, no requested changes, no merge conflicts, and a review state that satisfies branch protection.
   - If branch protection requires review, require `reviewDecision: APPROVED`.
   - If checks or mergeability are stale or unknown after refresh, put the PR under "Unknown" rather than calling it mergeable.
   - Link directly to each ready PR. Do not merge unless explicitly asked.

5. Report tersely.
   - Lead with action-needed PRs.
   - Then list ready-to-merge links only.
   - Then list unknown/stale PRs if any.
   - Omit no-action PRs unless the user asks for a full inventory.

## Provenance Filters

Use user-provided cohort language as a filter, not as a special case. Match against title, branch, body, labels, and repo when possible. Examples:

- Area PRs: package names, service labels, path names, domain words from the user's request.
- Release PRs: `release`, `backport`, version tags, release branches.
- Generated-code PRs: `generated`, `codegen`, `proto`, `wire`, generated file paths.
- Static-analysis or go-analysis PRs: `go-analysis`, `occult-go-analysis`, `static analysis`, `analyzer`, `detector`, `How found`, `Origin`, detector names.

When the filter is inferred, include one short sentence such as `Scoped to PRs whose title/body/branch matched static-analysis provenance.` Do not let an example cohort dominate the workflow.

## Output Shape

Use this shape by default:

```markdown
**Needs Action**
- PR title: reason. Link.

**Ready To Merge**
- Link
- Link

**Unknown**
- PR title: what could not be verified. Link.
```

If there is no action needed, say `No feedback needs action.` If there are no mergeable PRs, say `No PRs are ready to merge.`

## Query Patterns

Use GraphQL when scanning multiple PRs:

```bash
gh api graphql -F q='is:pr is:open author:USER org:ORG' -f query='
query($q:String!) {
  search(type: ISSUE, query: $q, first: 100) {
    nodes {
      ... on PullRequest {
        number
        title
        url
        state
        updatedAt
        isDraft
        headRefName
        bodyText
        reviewDecision
        mergeStateStatus
        mergeable
        statusCheckRollup { state }
        repository { nameWithOwner }
        comments(last:20) {
          nodes { author { login } bodyText createdAt updatedAt url }
        }
        reviews(last:50) {
          nodes { author { login } state submittedAt bodyText url }
        }
        reviewThreads(first:100) {
          nodes {
            isResolved
            comments(last:10) {
              nodes { author { login } bodyText createdAt updatedAt url path line }
            }
          }
        }
      }
    }
  }
}'
```

Use `--jq` or a JSON parser for filtering. Do not parse JSON with text matching alone.
