---
name: secrets-in-llm-output
description: "Reviewer persona for AI-generated code and logs: did the agent embed a real secret in a diff, commit message, log line, error message, comment, README, screenshot, or test fixture? With AI-mediated codebases this is now a distinct attack-surface class — agents see secrets from .env / config files / process env / tool output, and may reproduce them in proposed changes. Use after any agent-authored diff (claude-code, codex, opencode, pi, sqfan-spawned envs), after any agent session that ran with elevated access to env vars or secret stores, and as a pre-commit and pre-push gate. Triggers: AI-generated, agent diff, claude-code commit, codex commit, agent log, agent transcript, leaked secret in PR, agent secret exposure."
allowed-tools:
  - Read
  - Grep
  - Glob
  - Bash
---

# Secrets in LLM Output Review

Not generic secret-scanning on static codebases (gitleaks/trufflehog territory) and not runtime secret protection (`key-lifecycle-review`, `zeroize-audit`). This is the LLM-output layer: an agent with read access to secret material (`.env`, `~/.aws/credentials`, process env, tool output) may reproduce it verbatim in anything it writes. Treat agent output as having the access of the most-privileged context the agent ran in.

## Gate points

- Before merging any agent-authored PR
- Pre-commit / pre-push on agent-authored commits
- After any sqfan/squire ephemeral-env session with env-injected secrets
- Before sharing agent transcripts or logs externally
- Before publishing agent-produced artifacts: blog posts, demo recordings, screen captures, slides

## Scan surface

Per PR:
- Diff hunks of every changed file, plus the PR description itself
- Commit message bodies: `git log <base>..HEAD --format=%B` (full text, not just the first line)
- New test fixtures: `*.test.ts`, `testdata/`, `fixtures/`, `*.snap` (snapshots capture live output — frequent leak vector)
- Added/modified config: `.env*`, `*.yaml`, `*.toml`, `*.json`, `*.conf`
- README/docs additions (curl examples with real tokens); attached screenshots

Per session:
- Agent transcript files (claude-code session JSONL, codex session logs, pi session dirs)
- Artifacts under `~/.claude/projects/.../tool-results/`
- `/tmp/agent-context` and similar shared-context files

## Patterns

High-confidence:
- `sk-`, `sk_live_`, `sk_test_` (OpenAI, Stripe variants)
- `xoxb-`, `xoxp-`, `xoxa-`, `xoxs-` (Slack)
- `ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_` (GitHub)
- `AKIA[0-9A-Z]{16}` (AWS access key)
- `AIza[0-9A-Za-z\-_]{35}` (Google API key)
- `eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}` (JWT)
- `-----BEGIN (RSA |EC |DSA |OPENSSH |PRIVATE )?PRIVATE KEY-----`
- Long high-entropy hex/base64 near `token|secret|key|password|api_key|bearer|credential`

Pattern packs: `gitleaks detect --no-banner --staged`, or `git diff | gitleaks --pipe -`; trufflehog v3 verified mode authenticates hits to cut false positives.

LLM-specific:
- IdP issuer URL adjacent to a base64 chunk → likely OAuth tokens
- Verbatim narration with the value inline: "Here is the token I retrieved:", "Using this credential:", "From your .env:"
- Commit message says "removed the test value" — value sometimes still in the diff
- Newly added `console.log(process.env.X)` for sensitive X
- Test fixtures with realistic-looking IDs instead of `xxx`-style placeholders

## Transcript review

Walk the transcript: find tool calls that read sensitive paths (`.env`, `~/.aws/credentials`, `~/.ssh/`, `~/.config/op/`), then check every subsequent tool call that wrote text for data from those reads.

## On a verified hit

Rotate the credential immediately — assume it leaked even if the PR is closed unmerged. Then remediate the diff; rewrite history if the value reached a public branch.

## False-positive shapes

- Placeholders (`sk-XXXX`, `xoxb-FAKE`) — fine only with an explicit "example only" comment nearby
- `aws_access_key_id = "AKIAIOSFODNN7EXAMPLE"` (AWS docs example) — fine
- `// SAFE: not real` — trust-but-verify; confirm against the IdP if uncertain
- Known/transitional: the user's `xoxb-1381406198691-…` is known-exposed and mid-rotation — flag, don't panic

## Rationalizations to reject

- "It's a test value" — test values that look real usually are; the agent grabbed them from env because that was easiest
- "It's only in the commit message" — public the moment the branch is pushed

## Pre-commit hook starter

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit (or, better, via lefthook / pre-commit-framework)
if git diff --cached | gitleaks --pipe -no-banner --redact - 2>/dev/null; then
  echo "secret-leak heuristic fired; investigate before committing"
  exit 1
fi
```
