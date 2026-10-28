---
name: agent-sessions
description: "List, inspect, wait on, or focus agent sessions on this laptop and the user's Squire tasks (Claude Code, Codex, Grok, Squire) with their status, PRs and beads issues. Use when asked which agents are running or need input, to wait for a sibling agent to finish, or to open or focus another session."
---

# agent-sessions

Run `agent-sessions prime` first and follow what it prints: the commands,
the `review` JSON and its reconcile rules. The binary is
`~/.local/bin/agent-sessions`; source and README are in
`/Users/rch/repo/agent-sessions`.

`open` moves the user's focus; run it only when the user asks to see a
session. `review` makes one model call per session; name sessions or pass
`--repo` rather than reviewing everything.
