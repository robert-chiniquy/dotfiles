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

LLM-mediated coding pipelines have a new failure mode: the agent has
read-access to secret material (`.env`, `~/.aws/credentials`, env vars,
process tree) and may reproduce that material verbatim into a diff,
commit message, comment, README, log line, screenshot, or test
fixture. This skill catches that class on the way out.

## When to use

- Before merging any agent-authored PR
- After any sqfan / squire ephemeral-env session that had access to
  env-injected secrets
- As a pre-commit hook on agent-authored commits
- After any incident in which agent transcript files might have
  captured secrets (sharing transcripts externally, sending logs to a
  vendor, etc.)
- Before publishing any agent-produced artifact: blog post, demo
  recording, screen capture, slide deck

## When NOT to use

- Generic secret-detection in static codebases — that's a job for
  `gitleaks` / `trufflehog`; this skill is the LLM-output-specific
  layer
- Runtime secret protection — different concern (memory wiping,
  storage encryption) covered by `key-lifecycle-review` and
  `zeroize-audit`

## Core posture

Treat agent output as if it had the access of the most-privileged
context the agent ran in. If the agent could read your `.env`, assume
the agent could reproduce a key from `.env` in any of: code, comments,
commit message, README, error string, test value.

## What to scan

### Per-PR scan surface

- Every changed file's diff hunks
- The commit message bodies (full text, not just the first line)
- New test fixtures, especially `*.test.ts`, `testdata/`, `fixtures/`,
  `*.snap` (snapshot tests are a frequent leak vector — the snapshot
  captures live output)
- Configuration files added/modified: `.env*`, `*.yaml`, `*.toml`,
  `*.json`, `*.conf`
- README and docs additions (curl examples with real tokens, etc.)
- Screenshots if attached
- The PR description itself

### Per-session scan surface

- Agent transcript files (claude-code's session JSONL,
  codex's session logs, goose's logs if applicable, pi's session dirs)
- Any artifact written to `~/.claude/projects/.../tool-results/`
- `/tmp/agent-context` and similar shared-context files (verify they
  don't carry secret data inadvertently)

## Patterns

### High-confidence patterns

- `sk-`, `sk_live_`, `sk_test_` (OpenAI, Stripe variants)
- `xoxb-`, `xoxp-`, `xoxa-`, `xoxs-` (Slack)
- `ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_` (GitHub)
- `AKIA[0-9A-Z]{16}` (AWS access key)
- `AIza[0-9A-Za-z\-_]{35}` (Google API key)
- `eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}` (JWT)
- `-----BEGIN (RSA |EC |DSA |OPENSSH |PRIVATE )?PRIVATE KEY-----`
- Long high-entropy hex / base64 strings near field names
  `token|secret|key|password|api_key|bearer|credential`

### Pattern packs

- gitleaks-style ruleset is a strong starting point; can be invoked
  directly via `gitleaks detect --no-banner --staged` on the working
  tree, or against the diff with `git diff | gitleaks --pipe -`
- trufflehog v3 verified-secrets mode reduces false positives by
  attempting to authenticate the detected secret

### LLM-specific patterns to add

- An IdP issuer URL adjacent to a base64 chunk → likely OAuth tokens
- "Here is the token I retrieved:" / "Using this credential:" / "From
  your .env:" — verbatim narration with the value in line
- Commit-message references to "I removed the test value" — sometimes
  the value is still in the diff
- `console.log(process.env.X)` newly added — leak vector if X is sensitive
- Test fixtures with realistic-looking IDs that aren't `xxx`-style
  placeholders

## Review steps

1. **Diff scan**: `git diff` against the merge base; run gitleaks /
   trufflehog with verified mode; manually scan added lines for the
   patterns above
2. **Commit message scan**: `git log <base>..HEAD --format=%B`
3. **New-file scan**: `git diff --name-only --diff-filter=A` for
   every added file; pay special attention to fixtures/, testdata/,
   docs/
4. **Transcript scan** (if scoped to a session, not a PR): walk the
   agent's transcript JSONL; look for tool calls that read sensitive
   paths (`.env`, `~/.aws/credentials`, `~/.ssh/`, `~/.config/op/`),
   then look for any subsequent tool call that wrote text containing
   data from those reads
5. **Rotate immediately** on any verified hit — assume any exposed
   value has leaked even if the PR is closed without merge

## False-positive shapes

- `sk-XXXX` / `xoxb-FAKE` / placeholder secrets — fine, but require
  explicit "example only" comment nearby
- Test fixtures with documented `aws_access_key_id = "AKIAIOSFODNN7EXAMPLE"`
  (AWS docs example) — fine
- Live-looking values in a comment marked `// SAFE: not real` —
  trust-but-verify; confirm against the IdP if uncertain

## Output format

For each finding:

- Location (file:line; transcript:offset; commit:hash)
- Pattern class (OAuth / AWS / GitHub / private-key / JWT / generic-high-entropy)
- Confidence (high / medium / low)
- Recommended action: **rotate the credential**, then remediate the
  diff (rewrite history if value reached a public branch)

## Pre-commit hook starter

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit (or, better, via lefthook / pre-commit-framework)
# Block commits that smell like leaked secrets in agent-authored diffs.
if git diff --cached | gitleaks --pipe -no-banner --redact - 2>/dev/null; then
  echo "secret-leak heuristic fired; investigate before committing"
  exit 1
fi
```

## Rationalizations to reject

| Rationalization | Reality |
|---|---|
| "It's a test value" | Test values that look real are usually real; the agent grabbed them from env because that was easiest |
| "It's only in the commit message" | Commit messages are public the moment the branch is pushed |
| "I'll rotate later" | Rotate now |
| "The snapshot test will be updated soon" | The snapshot is in the diff right now |

## References

- gitleaks, trufflehog v3, detect-secrets
- GitHub's secret-scanning patterns reference
- OWASP Cheat Sheet on Secrets Management
- AI-specific incidents: 2024–2026 wave of agent-authored PRs leaking
  vendor tokens — search github.com/secret-scanning leak reports

## Status

**v0.1 draft** — pattern catalog covers the common high-confidence
shapes. Expansion: codebase-specific allowlist (e.g., the user's
`xoxb-1381406198691-…` is known and being rotated; transitional period
should flag it but not panic); per-agent transcript-format parsers
for claude-code, codex, opencode, pi; integration into
the `disk-emergency` style runbook for "secret leaked, here is the
recovery flow."
