# pr-status

Check status of your open pull requests: comments, reviews, CI failures, recent activity.

---

## Quick Commands

### List your open PRs

```bash
gh pr list --author @me --state open
```

### Check all PR activity at once

```bash
gh pr list --author @me --state open --json number,title,url,reviewDecision,statusCheckRollup,comments,updatedAt | jq '.[] | {number, title, reviewDecision, checks: (.statusCheckRollup | map(select(.conclusion != "SUCCESS")) | length), comments: (.comments | length), updatedAt}'
```

---

## Detailed PR Status

### Get PR details with checks and reviews

```bash
# Replace OWNER/REPO and PR_NUMBER
gh pr view PR_NUMBER --repo OWNER/REPO --json title,state,reviewDecision,statusCheckRollup,reviews,comments
```

### Check CI status for a PR

```bash
gh pr checks PR_NUMBER --repo OWNER/REPO
```

### Get recent comments on a PR

```bash
gh api repos/OWNER/REPO/issues/PR_NUMBER/comments --jq '.[-5:] | .[] | "\(.created_at) @\(.user.login): \(.body | split("\n")[0])"'
```

### Get review comments (on code)

```bash
gh api repos/OWNER/REPO/pulls/PR_NUMBER/comments --jq '.[-5:] | .[] | "\(.created_at) @\(.user.login) on \(.path):\(.line): \(.body | split("\n")[0])"'
```

---

## Batch Status Check

### All your PRs with failures

```bash
gh pr list --author @me --state open --json number,title,headRepository,statusCheckRollup | jq '.[] | select(.statusCheckRollup | any(.conclusion == "FAILURE")) | {number, title, repo: .headRepository.name, failed: [.statusCheckRollup[] | select(.conclusion == "FAILURE") | .name]}'
```

### PRs awaiting your response

```bash
# PRs with new comments since you last pushed
gh pr list --author @me --state open --json number,title,url,comments,commits | jq '.[] | select(.comments | length > 0) | {number, title, url, comment_count: (.comments | length)}'
```

### PRs ready to merge

```bash
gh pr list --author @me --state open --json number,title,reviewDecision,statusCheckRollup | jq '.[] | select(.reviewDecision == "APPROVED" and (.statusCheckRollup | all(.conclusion == "SUCCESS" or .conclusion == "SKIPPED"))) | {number, title}'
```

---

## Review Status

### Check review state

```bash
gh pr view PR_NUMBER --repo OWNER/REPO --json reviews --jq '.reviews | group_by(.author.login) | map({reviewer: .[0].author.login, state: .[-1].state})'
```

Review states:
- `APPROVED` - Ready to merge
- `CHANGES_REQUESTED` - Needs work
- `COMMENTED` - Feedback given, not blocking
- `PENDING` - Review in progress

---

## Watch for Updates

### Recent activity across all your PRs

```bash
gh pr list --author @me --state open --json number,title,updatedAt,url | jq -r 'sort_by(.updatedAt) | reverse | .[:5] | .[] | "\(.updatedAt | split("T")[0]) #\(.number) \(.title)"'
```

---

## Common Workflows

### Morning PR check

```bash
# 1. List all open PRs
gh pr list --author @me --state open

# 2. Check for failures
gh pr list --author @me --state open --json number,statusCheckRollup | jq '.[] | select(.statusCheckRollup | any(.conclusion == "FAILURE")) | .number'

# 3. Check for new comments (PRs updated in last 24h)
gh pr list --author @me --state open --json number,title,updatedAt | jq --arg cutoff "$(date -v-1d +%Y-%m-%dT%H:%M:%SZ)" '.[] | select(.updatedAt > $cutoff) | {number, title}'
```

### Respond to review

```bash
# See what reviewer said
gh pr view PR_NUMBER --comments

# See code comments
gh api repos/OWNER/REPO/pulls/PR_NUMBER/comments --jq '.[] | "\(.path):\(.line) - \(.body)"'
```

---

## GitHub MCP Integration

If GitHub MCP server is connected, these tools may be available:

- `mcp__github__get_pull_request` - Get PR details
- `mcp__github__list_pull_requests` - List PRs
- `mcp__github__get_pull_request_comments` - Get comments
- `mcp__github__get_pull_request_reviews` - Get reviews

Check available tools with ToolSearch if MCP is configured.

---

## Troubleshooting

### Auth issues

```bash
gh auth status
gh auth login  # if needed
```

### Rate limiting

```bash
gh api rate_limit --jq '.rate | "Used \(.used)/\(.limit), resets \(.reset | strftime("%H:%M"))"'
```
