---
name: multi-agent-coordination
description: |
  Coordinate multiple agents working simultaneously in a multi-subproject
  repository via INBOX-based communication. Use when multiple agents work
  in the same repo, when setting up COORDINATOR role, or when agents need
  territory assignments and collision prevention.
---

# Multi-Agent Coordination Among Subprojects

When multiple agents work simultaneously in a multi-subproject repository, they must coordinate to avoid file collisions. Uses INBOX-based asynchronous communication with a COORDINATOR role.

Key concepts: Agent identity (mandatory name prefix), COORDINATOR role (allocates work, tracks territories, responds to blockers, owns git checkpoints), INBOX-based communication (append-only, async), territory rules (exclusive ownership, no cross-boundary edits), mandatory progress reports.

COORDINATOR operations: autonomous headcount management, agent interviews before assignment, batch assignments to reduce idle time, background INBOX monitoring, periodic progress updates to user.
