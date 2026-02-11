# GitHub Skills Index

Skills for working with GitHub via CLI and MCP.

---

## Available Skills

| File | Purpose |
|------|---------|
| `pr-status.md` | Check your open PRs: comments, reviews, CI failures, activity |

---

## Quick Reference

### Your open PRs
```bash
gh pr list --author @me --state open
```

### PRs with CI failures
```bash
gh pr list --author @me --state open --json number,statusCheckRollup | jq '.[] | select(.statusCheckRollup | any(.conclusion == "FAILURE")) | .number'
```

### Recent comments on a PR
```bash
gh api repos/OWNER/REPO/issues/PR_NUMBER/comments --jq '.[-5:] | .[] | "@\(.user.login): \(.body | split("\n")[0])"'
```

### PR ready to merge (approved + green)
```bash
gh pr list --author @me --state open --json number,title,reviewDecision,statusCheckRollup | jq '.[] | select(.reviewDecision == "APPROVED" and (.statusCheckRollup | all(.conclusion == "SUCCESS" or .conclusion == "SKIPPED")))'
```
