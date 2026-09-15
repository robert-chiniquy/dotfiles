---
name: sleep-report
description: >
  Report laptop sleep at the start of every turn. Always-on —
  run `sleep-report --since` before other work.
---

# sleep-report

## Common Mistakes

1. **Starting a turn without `sleep-report --since`** — MUST run it before other work. If it prints `SLEEP <dur>`, the laptop slept; do not treat that wall gap as a hang or a missed poll.
2. **Ignoring `SLEEP` lines from `pr-watch`** — a poll that overran is host sleep, not stuck GitHub.
3. **Printing `SLEEP` to the user** — advisory for the agent only. MUST NOT report sleep durations in chat.
4. **Treating a silent second `--since` as a hang** — a look within 10 seconds of the previous `--since` or `--hook` exits 0 without sysctl. Still run `--since` at turn start.

`sleep-report prime` prints the contract.
