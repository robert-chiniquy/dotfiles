# Multi-Agent Coordination Rules (Removed)

Removed from `~/.claude/CLAUDE.md` on 2026-02-08.

## Why Removed

These rules implemented a file-based INBOX/COORDINATOR pattern for multi-agent coordination. Claude Code now has native agent teams (CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1) with built-in messaging, task lists, and coordination. The INBOX/COORDINATOR file-polling approach is superseded.

## What Replaced It

Native agent teams with SendMessage, TaskCreate/TaskUpdate, and TeamCreate tools.

## The Rules

### Check INBOX at phase boundaries
In any project that has an `INBOX/` directory, check it at stopping points and phase boundaries. INBOXes contain notes from neighbors, new goals, or external inputs that may affect next steps. Don't ignore notes just because they arrived mid-work.

### Report discovered TODOs to coordinators
In multi-agent projects with a COORDINATOR INBOX, report important TODOs you discover during work. Don't assume TODOs are "just informational" - they may be actual implementation tasks that need assignment. When you find work that's outside your territory or that others should know about, drop a note in the COORDINATOR INBOX with: (1) what the TODO is, (2) its priority/urgency, (3) which territory it affects, (4) current status. This keeps the coordinator aware of work that needs assignment.

### Ask COORDINATOR for help when blocked
When you hit a blocker (missing dependencies, unclear requirements, need expertise outside your domain, architectural decisions needed, waiting on external resources), don't just stop or spin. Write a note to the COORDINATOR INBOX explaining: (1) what you're blocked on, (2) what you've tried, (3) what you need to unblock. The coordinator can reassign work, bring in specialists, or escalate. Blocking silently wastes time.

### COORDINATOR is your boss in multi-agent projects
When a COORDINATOR is present, they are your manager. Poll for input from COORDINATOR as instructed. When you have questions about priorities, scope, or direction, ask the COORDINATOR via INBOX - not the user directly. The COORDINATOR can escalate to the user if needed. Follow COORDINATOR's assignments and report progress to them. This hierarchy keeps multi-agent work coordinated without the user needing to manage each agent individually.

### Read INBOX items before processing
You cannot move items to processed/ without reading them first. INBOX items may contain important information, blockers, or action items that require acknowledgment.

## Antipattern

File-based inter-agent communication (polling directories, writing notes to INBOX/, moving to processed/) when the platform provides native messaging primitives. The file approach had race conditions, no delivery guarantees, and required explicit polling instructions in every agent's config.
