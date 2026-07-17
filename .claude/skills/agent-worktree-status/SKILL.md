---
name: agent-worktree-status
description: Check the progress and liveness of background agent worktrees with one pre-authorized command instead of ad hoc shell. Use when monitoring dispatched background agents, when the user asks for interim status on agent work, when deciding whether an agent is stalled, or before killing/restarting an agent. Triggers on - agent status, worktree status, is the agent doing anything, agent progress, agent stalled, liveness check.
---

# agent-worktree-status

One pre-authorized, read-only command reports every agent worktree's
progress. Use it instead of composing ad hoc `git -C`/`ls`/`ps`
one-liners, which each cost the user a permission prompt.

```
/Users/rch/repo/dotfiles/scripts/agent-wt-status.sh [repo-root]
```

Default repo-root is `/Users/rch/repo/occult`; pass another repo root
to inspect its `.claude/worktrees`. Already allowlisted in occult's
`.claude/settings.local.json` (exact and any-args forms). For a new
repo, add the same two allow entries once.

## What it reports, per worktree

- branch, commits ahead of origin/main, dirty file count
- last commit subject
- newest non-.git file with age in minutes — the liveness signal
- the last line of `PROGRESS.md` if the agent maintains one
- plus any running go test/build processes

## Interpreting it

- `newest-file` minutes old + growing commits: agent is healthy.
- No filesystem trace at all is NOT proof of death: a read-heavy
  design phase (large-file reading, planning) legitimately leaves
  zero trace for 30-60+ minutes. A healthy agent was once killed on
  exactly this misreading. Before killing, ping the agent with
  SendMessage and give it a real deadline; kill only if the ping is
  never consumed.
- Worktrees whose branches are already merged upstream are prune
  candidates; ask the user before removing.

## Proactive monitoring (no polling, no user asks)

For a long-running agent, don't make the user ask for status: arm a
persistent Monitor on its worktree that emits an event on stage
changes and new commits. Pair it with the agent's own completion
notification and progress arrives unprompted.

```
Monitor (persistent: true), command:
  /Users/rch/repo/dotfiles/scripts/agent-wt-watch.sh <worktree-name> [repo-root] [heartbeat-secs]
```

The script emits one line per stage change (PROGRESS.md tail) and per
new commit, plus a terminal WORKTREE GONE line so silence never
masquerades as progress. With heartbeat-secs (e.g. 600), it also emits
a HEARTBEAT summary line on that cadence regardless of change — use
when the user asks for periodic progress reports; relay each heartbeat
with a one-line interpretation. Change events reset the heartbeat
clock. 60s local polling, read-only. Stop with TaskStop when the
agent's final report lands. This only has signal if the agent
maintains PROGRESS.md and commits per stage — see the companion
conventions below.

Ground-truth caveat: the status script's newest-file field and the
harness's batched diagnostics can lag live edits by minutes. When they
disagree with expectations, spot-check specific file mtimes (stat)
before concluding an agent is stalled.

## Companion conventions for dispatched agents

Put these in long-running implementation briefs so the script has
signal to read:

- Commit each stage as soon as it compiles and its focused tests pass
  (early, visible commits — progress is never invisible).
- Append a one-line stage note to `PROGRESS.md` in the worktree at
  every stage switch.
