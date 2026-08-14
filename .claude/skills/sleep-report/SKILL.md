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

`sleep-report prime` prints the contract.
