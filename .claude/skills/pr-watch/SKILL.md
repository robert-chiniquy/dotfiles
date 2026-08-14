---
name: pr-watch
description: >
  Catch up on GitHub PR events without missing transitions. Always-on —
  after any prior look at a PR, use `pr-watch --since` instead of
  `--once` or `gh pr view`.
---

# pr-watch

## Common Mistakes

1. **`--once` or `gh pr view` as the second look** — that is a snapshot. It hides the transition. MUST `pr-watch --since owner/repo#N`.
2. **`cmd &` / sleep-poll** — MUST `pr-watch --until action` or wrap it with the harness monitor.
3. **Assuming one cursor is global** — state is per cwd + repo + PR in `~/.config/pr-watch/`.
4. **Ignoring `NEXT` lines** — they are the procedure for that event. MUST follow them (fix Changes Requested threads, then watch CI again).

`pr-watch prime` prints the contract.

Catch-up: `pr-watch --since owner/repo#N`
Wait: `pr-watch --until action owner/repo#N`
Catch-up then wait: `pr-watch --since --until action owner/repo#N`
