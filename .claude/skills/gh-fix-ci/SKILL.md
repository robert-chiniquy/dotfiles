---
name: "gh-fix-ci"
description: "Use when a user asks to debug or fix failing GitHub PR checks that run in GitHub Actions; use `gh` to inspect checks and logs, summarize failure context, draft a fix plan, and implement mechanical CI fixes on this session's PRs without a second plan approval. If Actions logs exist, use them. If the failing check is Buildkite or logs are missing, reproduce the repo lint/fmt/vendor steps locally and fix."
---


# Gh Pr Checks Plan Fix

## Overview

Use gh to locate failing PR checks, fetch GitHub Actions logs for actionable failures, summarize the failure snippet, then apply mechanical CI fixes on this session's PRs without a second plan approval.

Prereq: authenticate with the standard GitHub CLI once (for example, run `gh auth login`), then confirm with `gh auth status` (repo + workflow scopes are typically required).

## Inputs

- `repo`: path inside the repo (default `.`)
- `pr`: PR number or URL (optional; defaults to current branch PR)
- `gh` authentication for the repo host

## Quick start

- `gh pr checks <pr>`
- `gh run view <run_id> --log` for Actions logs
- GraphQL for fields `gh pr view --json` does not have. Never python or perl.

## Workflow

1. Verify gh authentication.
   - Run `gh auth status` in the repo.
   - If unauthenticated, ask the user to run `gh auth login` (ensuring repo + workflow scopes) before proceeding.
2. Resolve the PR.
   - Prefer the current branch PR: `gh pr view --json number,url`.
   - If the user provides a PR number or URL, use that directly.
3. Inspect failing checks with `gh pr checks`, `gh run view`, and GraphQL only. Never python or perl.
   - `gh pr checks <pr> --json name,state,bucket,link,startedAt,completedAt,workflow`
     - If a field is rejected, rerun with the available fields reported by `gh`.
   - For each failing check, extract the run id from `detailsUrl` and run:
     - `gh run view <run_id> --json name,workflowName,conclusion,status,url,event,headBranch,headSha`
     - `gh run view <run_id> --log`
   - If the run log says it is still in progress, fetch job logs directly:
     - `gh api "/repos/<owner>/<repo>/actions/jobs/<job_id>/logs" > "<path>"`
4. If Actions logs exist, use them. If the failing check is Buildkite or logs are missing, reproduce the repo lint/fmt/vendor steps locally and fix.
5. Summarize failures for the user.
   - Provide the failing check name, run URL (if any), and a concise log snippet.
   - Call out missing logs explicitly.
6. Mechanical CI fixes on this session's PRs run without a second plan approval.
7. Recheck status.
   - After changes, re-run the relevant tests and `gh pr checks`.

## Bundled Resources

Do not run the bundled python script. Inspect with `gh` only.
