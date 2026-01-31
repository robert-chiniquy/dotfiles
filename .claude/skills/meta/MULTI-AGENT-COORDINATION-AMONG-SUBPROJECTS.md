# Multi-Agent Coordination Among Subprojects

When multiple agents work simultaneously in a multi-subproject repository, they must coordinate to avoid file collisions. Collisions occur when two agents edit the same file - each sees the other's WIP as bugs or stale code.

## Agent Identity (MANDATORY)

**Every agent MUST prefix all communications to the user with their assigned name.**

Examples:
- `COORDINATOR: I've reviewed the INBOX and found 3 new reports.`
- `ALPHA: I've completed the initial_facts implementation.`
- `BETA: I'm blocked on baton-github - the mock server isn't responding.`
- `A17: The verify package tests are passing.`

This allows the user to immediately identify which agent is speaking when multiple terminals are open. Without this prefix, the user cannot tell agents apart.

**Format:** `NAME: <message>`

Names can be anything short and distinctive: ALPHA, BETA, GAMMA, A17, CORE, etc. The COORDINATOR assigns names in the assignment file.

The name should appear at the start of every response, not just the first message in a conversation.

## The COORDINATOR Role

### INBOX CHECK REMINDER (EVERY RESPONSE)

**CHECK YOUR INBOX NOW:** `cat ./scripts/coordinator-inbox.log`

Every response, check for new agent messages. Start the monitor if not running:
```bash
ps aux | grep coordinator-monitor | grep -v grep || nohup ./scripts/coordinator-monitor.sh >> ./scripts/coordinator-inbox.log 2>&1 &
```

Agents are waiting. Don't forget. This is your primary job.

### NEVER STOP POLLING

**Polling continues until the project is complete.** Even when agents have large batch assignments and won't report for a while, keep checking:

1. Check INBOX at least every few minutes
2. After context compaction, IMMEDIATELY resume polling
3. The project is not done until all agents report completion
4. "Quiet" doesn't mean "done" - agents may be blocked and waiting

**Post-compaction recovery:** If you've just been restored from compaction, your first action is:
```bash
ps aux | grep coordinator-monitor | grep -v grep || nohup ./scripts/coordinator-monitor.sh >> ./scripts/coordinator-inbox.log 2>&1 &
ls /Users/rch/repo/research/spike-classifiers/INBOX/*.md 2>/dev/null | grep -v README
```

Then read any messages and resume coordination.

### AGENT TIMEOUT FALLBACK

**If all agents appear to have timed out (no reports for extended period), COORDINATOR carries on the mission:**

1. **Checkpoint commit first** - Save all current progress before doing any work
2. **Check for ongoing changes** - Look for uncommitted changes that might indicate an agent is still working:
   ```bash
   git status --short
   ```
3. **If no activity detected** - Pick up the highest-priority incomplete task yourself
4. **Announce to all agents** - Write a notice to each agent's INBOX saying what you're doing (prevents collision if they come back)
5. **Work in the agent's territory** - Follow the same rules agents follow
6. **Resume coordination when agents return** - If an agent reports back, hand work back to them

This prevents the project from stalling when agents timeout. COORDINATOR is the fallback worker, not just a manager.

### COORDINATOR WORK ANNOUNCEMENTS

**Whenever COORDINATOR takes on a work item directly, announce it to ALL agents:**

Write to each agent's INBOX:
```markdown
# Notice: COORDINATOR Taking Work Item

**Date:** YYYY-MM-DD
**Item:** [description of what you're doing]
**Territory:** [which agent's territory this touches]
**Reason:** [why COORDINATOR is doing this - timeout, urgent, etc.]

If you return and see this, check with COORDINATOR before resuming work in this area.

-- COORDINATOR
```

This prevents:
- Collision if an agent returns mid-work
- Confusion about who owns what
- Duplicate effort

---

**COORDINATOR** is a dedicated agent that:
- **Interviews agents** to understand their specialties and warm context before assigning work
- Allocates work to prevent file overlap, matching tasks to agent strengths
- Tracks which agent owns which files/directories
- Responds to blockers and reallocation requests
- Maintains situational awareness across all agents
- **Approves agent requests** (territory changes, permission to cross boundaries, etc.)
- **Owns git checkpoint commits** for the project

The COORDINATOR does not do implementation work. It coordinates.

### COORDINATOR as Approval Authority (CRITICAL)

**NEVER ask the user for permission. Ask COORDINATOR.**

This is not optional. The user has delegated operational authority to COORDINATOR. Agents who ask the user directly are violating the process and wasting the user's time.

Agents should request permission from **COORDINATOR, not the user**, for:
- Temporary access to another agent's territory
- Scope clarifications
- Priority decisions
- Any operational question within the project
- **Coordination issues**: blockers, cross-project dependencies, resource conflicts, unclear requirements, build failures affecting multiple agents

COORDINATOR can approve requests as long as:
1. The agent is following the global project process
2. The request is within the scope COORDINATOR has defined
3. It doesn't conflict with another agent's work

**Escalate to user only** when COORDINATOR cannot resolve (e.g., strategic direction, resource allocation, scope changes beyond current assignments).

### COORDINATOR Escalation to User

When an agent asks COORDINATOR a permission question that COORDINATOR is unsure about, COORDINATOR escalates to the user. The flow is:
1. Agent asks COORDINATOR
2. COORDINATOR evaluates the request
3. If within scope and clear: COORDINATOR approves
4. If unclear or outside scope: COORDINATOR asks user
5. User responds to COORDINATOR
6. COORDINATOR relays answer to agent

**Agents never ask user directly.** This is non-negotiable.

### Blanket Permission Pattern

Agents should ask the user ONCE for blanket permission:

> "[AGENT_NAME]: COORDINATOR has assigned me to [territory]. Do I have your blanket permission to proceed with any tasks COORDINATOR assigns within this scope, without checking back with you each time?"

Once the user grants blanket permission:
- Agent follows COORDINATOR's direction without asking user
- COORDINATOR approves operational requests
- User is only involved for escalations

This prevents agents from repeatedly asking the user for permission on routine operations.

### Agent Interviews

Before assigning work, COORDINATOR should interview agents to understand their specialty:

**Interview question:** "What is your specialty?"

Agents should describe:
1. What context they have warm (recent work, files edited)
2. Domain knowledge accumulated this session
3. Technologies or patterns they've been deep in
4. Current work in progress (if any)

**Why this matters:**
- Agents with warm context on a topic work faster
- Mismatched assignments waste time on ramp-up
- Agents may have discovered things that affect task priority

**When to interview:**
- When an agent first joins or reports in
- Before assigning new work to an idle agent
- When reassigning work after a blocker

The interview response goes to COORDINATOR's INBOX. COORDINATOR then assigns work that matches the agent's strengths.

### Git Checkpoint Commits

COORDINATOR is responsible for creating git checkpoint commits at:
- Phase boundaries (when significant milestones complete)
- Before major changes that could be risky
- When agents report completion of substantial work
- Periodically to preserve progress

Commit message format:
```
<subproject>: <brief description of changes>

- Agent ALPHA: <what they completed>
- Agent BETA: <what they completed>
- Agent DELTA: <what they completed>
```

This ensures work is preserved and attributable.

### Background INBOX Monitoring

COORDINATOR should run a background process to monitor its INBOX while remaining conversational with the user:

```bash
# Monitor script lives in project scripts/ directory
# Start it:
nohup ./scripts/coordinator-monitor.sh >> ./scripts/coordinator-inbox.log 2>&1 &

# Check for new messages:
cat ./scripts/coordinator-inbox.log
```

The script and log live in the project's `scripts/` directory, not `/tmp/` (which is volatile).

**IMPORTANT: Self-reminder loop.** After checking the log and responding to messages, COORDINATOR should:
1. Check if the monitor is still running: `ps aux | grep coordinator-monitor | grep -v grep`
2. Restart if needed
3. Set a mental reminder to check again in ~5-10 minutes

This is easy to forget. Build the habit: after every user interaction pause, check the log.

**IMPORTANT: Always notify user when polling INBOX.** When COORDINATOR checks the INBOX (either via the log or directly), always tell the user explicitly:
- "Checking INBOX..." before looking
- Summary of what was found (or "No new messages")
- Any actions taken in response

The user needs visibility into COORDINATOR's monitoring activity. Silent polling is not acceptable.

This allows COORDINATOR to be interrupt-driven (responsive to user) while passively monitoring agent communications.

### Periodic Progress Updates to User

COORDINATOR should proactively report project-level progress to the user every ~10 minutes (or at natural pauses).

**Every summary MUST include progress against STRATEGY** (or the top-level plan document). This is mandatory, not optional.

Structure:
1. **STRATEGY Progress** - Current phase, what's complete, what remains (ALWAYS FIRST)
2. **Agent status** - What is each agent working on? Any blockers?
3. **Recent completions** - What got done since last update?
4. **Next milestones** - What's the immediate path forward?

Reference the project's STRATEGY.md (or equivalent) as the source of truth for overall progress. COORDINATOR maintains situational awareness so the user doesn't have to poll each agent individually.

## INBOX-Based Communication

Every agent communicates via INBOX directories. This is append-only, asynchronous messaging.

### COORDINATOR's INBOX

The **top-level INBOX** (`/INBOX/` at repository root) belongs to COORDINATOR.

Agents send messages here:
- Progress reports
- Blocker notifications
- Reallocation requests
- Task completion notices

### Agent INBOXes

Each subproject or territory has its own INBOX:
- `17-model-verification/INBOX/` - for agents working there
- `baton-regression/INBOX/` - for connector agents
- `18-go-afl/INBOX/` - for static analysis agents

COORDINATOR sends assignments and responses to these INBOXes.

## Message Formats

### Assignment (COORDINATOR -> Agent)

```markdown
# Assignment: AGENT-NAME

**From:** COORDINATOR
**Date:** YYYY-MM-DD
**Priority:** high | medium | low

## Your Territory

You own exclusively:
- path/to/directory/*
- path/to/specific/files.go

## Do NOT Touch

- Directories owned by other agents
- Shared infrastructure (unless cleared)

## Current Tasks

### Task 1: Name
Description and acceptance criteria

## Communication

- Report to: /INBOX/report_AGENT-NAME_*.md
- I respond in: your-territory/INBOX/
```

### Progress Report (Agent -> COORDINATOR)

```markdown
# Progress Report: AGENT-NAME

**Date:** YYYY-MM-DD HH:MM
**INBOX:** /full/path/to/my/INBOX/
**Period:** What timeframe this covers

## Completed
- Item 1
- Item 2

## In Progress
- Item with current state

## Blocked
- Item and what's blocking it

## Files Modified
- List all files touched (for collision tracking)
```

**IMPORTANT:** Always include your INBOX path so COORDINATOR knows where to send responses.

### Request (Agent -> COORDINATOR)

```markdown
# Request: AGENT-NAME

**Date:** YYYY-MM-DD HH:MM
**INBOX:** /full/path/to/my/INBOX/
**Type:** reallocation | clarification | permission

## Request

What you need from COORDINATOR.

## Context

Why you need it.
```

**IMPORTANT:** Always include your INBOX path so COORDINATOR knows where to send responses.

## Territory Rules

1. **Exclusive ownership** - Each file/directory has exactly one owner
2. **No cross-boundary edits** - Don't touch files outside your territory
3. **Request before crossing** - If you must edit outside, ask COORDINATOR first
4. **Geographic separation** - Prefer assigning whole directories, not individual files

## Collision Prevention

### Good Patterns

- Agent A owns `subproject-1/*`, Agent B owns `subproject-2/*`
- Agent A owns connectors A-M, Agent B owns connectors N-Z
- COORDINATOR owns no implementation files

### Bad Patterns

- Two agents "sharing" a package
- Agent edits "just one small file" in another's territory
- No clear ownership boundaries

## Agent Lifecycle

### Starting Work

1. Check your INBOX for assignments
2. Read the assignment completely
3. Verify you understand your territory boundaries
4. Begin work only on owned files

### During Work

1. Stay within territory
2. Report blockers immediately (don't wait)
3. Drop progress reports periodically
4. If you need something outside territory, request it
5. **Ask COORDINATOR about any coordination issue** - blockers, cross-project dependencies, scope questions, resource conflicts, unclear requirements, build failures affecting multiple agents - all go to COORDINATOR, not the user

### Completing Work

1. Send completion notice to COORDINATOR INBOX
2. List all files modified
3. Note any follow-up work needed
4. **Request next assignment from COORDINATOR** - Don't just wait passively. File a request to COORDINATOR's INBOX asking for your next task. COORDINATOR may not know you're idle.
5. Process your INBOX (move handled messages to `processed/`)

### Waiting for COORDINATOR Response

When you're blocked and need COORDINATOR's input, or when you're between assignments, use the wait script to poll your INBOX:

```bash
./scripts/agent-wait-for-coordinator.sh YOUR_INBOX_PATH [timeout_minutes]
```

**Examples:**
```bash
./scripts/agent-wait-for-coordinator.sh 17-model-verification/INBOX 10
./scripts/agent-wait-for-coordinator.sh baton-regression/INBOX 15
./scripts/agent-wait-for-coordinator.sh 18-go-afl/INBOX 5
```

**How it works:**
1. Creates a timestamp marker in your INBOX
2. Polls every 30 seconds for new `.md` files
3. When a new file appears (COORDINATOR's response), prints its contents and exits
4. Times out after the specified minutes (default: 10)

**When to use:**
- After sending a blocker report, while waiting for direction
- After completing all assigned work AND requesting next assignment
- Any time you need COORDINATOR input to proceed

**IMPORTANT:** Before waiting, ensure you have ASKED for what you need. Don't just wait passively - file a request to COORDINATOR's INBOX first, then wait for response.

**The idea:** Rather than spinning doing nothing or constantly checking manually, the script lets you "sleep" efficiently until COORDINATOR responds. This is especially useful when you're blocked and can't make progress without direction.

**Post-compaction reminder:** When the script detects a response, it also prints a reminder of the key coordination protocols. This helps agents who may have lost context during compaction remember the rules.

## Mandatory Progress Reports

**Once an agent receives an assignment from COORDINATOR, progress reports are MANDATORY.**

### When to Report

You MUST deliver a progress report to COORDINATOR's INBOX (`/INBOX/`) at:

1. **Task completion** - When you finish a task in your assignment
2. **Phase boundaries** - When transitioning between phases of work
3. **Session end** - Before context compaction or ending your session
4. **Blockers** - Immediately when blocked (don't wait for a boundary)
5. **Blocker fixes** - When you fix something that was blocking another agent

### Blocker Fix Notification (IMPORTANT)

When you fix a bug or issue that was blocking another agent:
- **Notify COORDINATOR** (not just the blocked agent)
- COORDINATOR tracks overall project state and needs to know blockers are resolved
- If you only tell the affected agent, COORDINATOR still thinks there's a blocker

**Wrong:** DELTA fixes build break, tells ALPHA directly, COORDINATOR still thinks ALPHA is blocked.

**Right:** DELTA fixes build break, tells ALPHA AND drops a note to `/INBOX/` saying "Fixed the build break that was blocking ALPHA."

### Why This Matters

COORDINATOR cannot see your work directly. Without reports:
- COORDINATOR doesn't know if you're stuck
- Work may be reassigned to someone else
- Collision risks go undetected
- The project loses visibility into progress

### Minimum Report Content

Every report MUST include:
1. What you completed
2. What you're working on now
3. Files you modified (full paths)
4. Any blockers or questions

### Report Frequency

- For short tasks: Report on completion
- For multi-hour work: Report at least once per significant milestone
- When in doubt: Report more, not less

**Silence is not acceptable.** If COORDINATOR doesn't hear from you, it assumes something is wrong.

## COORDINATOR Operations

### Assigning Work

1. **Interview agents first** - Before assigning work, ask agents "What is your specialty?" to understand:
   - What context they have warm (recent work, domain knowledge)
   - What technologies or patterns they've been deep in
   - Any learnings from their current session
   This helps match agents to tasks they'll be efficient at.

2. Review available work (TODOs, INBOX goals, etc.)
3. Divide into non-overlapping territories based on agent strengths
4. Write assignment files to agent INBOXes
5. Track which agent owns what

### Batch Assignments (Reduce Idle Time)

**Assign larger batches of work so agents spend less time waiting.**

Instead of assigning one task and waiting for a report, give agents multiple tasks upfront:

**Bad pattern:**
```
COORDINATOR: Do Task A
AGENT: Done with A, what next?
COORDINATOR: Do Task B
AGENT: Done with B, what next?
(agent idle while waiting for each response)
```

**Good pattern:**
```
COORDINATOR: Do Tasks A, B, C, D. Report when all done or if blocked.
AGENT: (works through A, B, C, D without waiting)
AGENT: Batch complete. A-C passed, D blocked on X.
```

**Benefits:**
- Agents stay busy longer between check-ins
- Fewer round-trips through INBOX
- COORDINATOR can focus on other work
- Agents develop momentum on related tasks

**What to batch:**
- Multiple connectors of the same type
- Sequential tasks in a pipeline
- Related fixes or features
- Coverage runs across similar targets

**When NOT to batch:**
- Exploratory/uncertain work where direction may change
- High-risk changes needing review between steps
- Work that may block other agents

Agents should still report immediately on blockers, but don't need to report after each sub-task.

### Handling Requests

1. Check COORDINATOR INBOX regularly
2. Respond promptly to blockers
3. Reallocate if needed
4. Update territory assignments
5. **Move processed messages to `INBOX/processed/`** - keeps INBOX clean

### INBOX Hygiene

Once a message has been fully handled (responded to, actioned, or acknowledged), move it:
```bash
mv INBOX/report_ALPHA_*.md INBOX/processed/
```

This keeps the INBOX clean so you can quickly see what's new vs what's been dealt with. Create the `processed/` subdirectory if it doesn't exist.

### File Move Safety

**ALWAYS check for name conflicts before using mv:**
```bash
# Check if destination exists first
[ -f INBOX/processed/report_ALPHA_file.md ] && echo "CONFLICT" || mv INBOX/report_ALPHA_file.md INBOX/processed/
```

If a file with the same name exists at the destination:
- Check if it's the same content (duplicate move attempt)
- If different content, rename with timestamp suffix: `report_ALPHA_file_1430.md`
- Never overwrite without checking

### Maintaining Awareness

1. Read progress reports
2. Track modified files for collision risk
3. Identify when work streams might converge
4. Proactively reassign before collisions

## Example Territory Division

For a repository with 50+ connectors:

| Agent | Territory |
|-------|-----------|
| AGENT-ALPHA | Core framework (verify/*.go) |
| AGENT-BETA | Connectors A-K (baton-asana through baton-kubernetes) |
| AGENT-GAMMA | Connectors L-Z (baton-linear through baton-zoom) |

This ensures zero overlap. Each agent can work at full speed without coordination overhead.

## State Synchronization and Confusion Clarification

**If COORDINATOR's understanding contradicts what you know to be true, clarify immediately.**

Context compaction can cause state drift between COORDINATOR and agents. Common scenarios:
- COORDINATOR assigns work you already completed
- COORDINATOR references a blocker you already resolved
- COORDINATOR's timeline differs from your records

**When confusion occurs:**
1. Send a `clarification_AGENTNAME_*.md` message to COORDINATOR's INBOX
2. State what you know to be true with evidence (report timestamps, file paths, test results)
3. Ask for confirmation or correction
4. Don't proceed with conflicting instructions until resolved

**Example clarification:**
```markdown
# Clarification: DELTA -> COORDINATOR

**Date:** YYYY-MM-DD HH:MM
**Re:** Task assignment confusion

## What I Know

According to my report `INBOX/report_DELTA_*.md` at HH:MM:
- Task X is COMPLETE
- Tests passing (make 18-test)
- Files modified: path/to/file.go

## Apparent Confusion

I received an assignment for Task X after my completion report.

## Question

Can you confirm Task X status? Should I proceed to next task?

-- DELTA
```

**Why this matters:**
- Silent confusion wastes compute (redoing done work)
- Conflicting state leads to coordination failures
- Early clarification is cheaper than late discovery

## When Boundaries Must Cross

Sometimes work genuinely spans territories. Options:

1. **Sequential handoff** - Agent A completes, Agent B takes over
2. **COORDINATOR implements** - Small cross-cutting changes done by COORDINATOR
3. **Territory reassignment** - Temporarily expand one agent's territory
4. **Interface agreement** - Agents agree on interface, implement independently

Always prefer clear boundaries over "just this once" sharing.
