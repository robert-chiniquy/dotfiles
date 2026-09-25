---
name: pr-watch
description: >
  Catch up on GitHub PR events without missing transitions. Always-on —
  after any prior look at a PR, use `pr-watch --since` instead of
  `--once` or `gh pr view`.
---

# pr-watch

## Common Mistakes

1. **`--once` or `gh pr view` as the second look.** That is a snapshot. It hides the transition. MUST `pr-watch --since owner/repo#N`. When squine MCP or CLI is connected, MUST `github_track` then `github_poll` first. pr-watch is only for PRs outside squine track sets.
2. **`cmd &` / sleep-poll** — MUST `pr-watch --until action` or wrap it with the harness monitor.
3. **Assuming one cursor is global** — state is per cwd + repo + PR in `~/.config/pr-watch/`.
4. **Ignoring `NEXT` lines** — they are the procedure for that event. MUST follow them (fix Changes Requested threads, then watch CI again).
5. **Asking a human before Copilot** — MUST `pr-watch request-copilot owner/repo#N` when COPILOT NONE, then wait for COPILOT REVIEWED. MUST NOT request a human review while COPILOT is NONE or REQUESTED. MUST NOT invent a reviewer login (it is `github-copilot`). Exception: resolved mode `skip` in `~/.config/pr-watch/config.yaml` (global `copilot:` or `cwd.<dir>.copilot`) emits `COPILOT SKIP` and does not require a Copilot review. `do not merge unless authorized` includes Claude.md standing grants: a qualifying pure-tests and/or pure-docs PR with CI green on the merge SHA is already authorized. Do not wait for Copilot or a human before merging those. Production comments, CI, config, engine, and stdlib are not that grant.
6. **Ignoring `SLEEP <dur>`** — the host slept; do not treat that gap as a hang.
7. **Ignoring `OUTAGE START` / `OUTAGE <dur>` / `OUTAGE ONGOING`** — GitHub fetches failed on the network. MUST NOT treat that as a hang or as CI. MUST NOT print OUTAGE durations to the user.

`pr-watch prime` prints the contract.

Catch-up: `pr-watch --since owner/repo#N`
Wait: `pr-watch --until action owner/repo#N`
Catch-up then wait: `pr-watch --since --until action owner/repo#N`
Request Copilot: `pr-watch request-copilot owner/repo#N`
Global Copilot mode: `pr-watch set copilot request|skip`
Cwd override: `pr-watch skip-copilot` / `pr-watch set copilot skip --cwd DIR`
