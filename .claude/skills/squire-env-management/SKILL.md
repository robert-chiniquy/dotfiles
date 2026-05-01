---
name: squire-env-management
description: >-
  Create and manage Squire ephemeral development environments for parallel agent
  work. Use when delegating implementation tasks to remote environments, creating
  fire-and-forget work sessions, or monitoring parallel agents. Triggers on:
  squire, ephemeral env, parallel agents, fire-and-forget, remote development.
---

# Squire Env Management

Squire provisions ephemeral dev environments with an in-env agent (OpenCode, not Claude Code). Use these for parallel, isolated work: each env gets its own container, services, and agent session.

## Core Commands

### Create an env (fire-and-forget)

```bash
squire new <name> --prompt "Implement feature X in pkg/foo"
```

The agent starts working immediately. `--prompt` is the instruction for OpenCode inside the env.

CLI flags: `--image`, `--no-open`, `--prompt`, `--timeout`

The API supports additional fields not yet exposed by the CLI: Model, Flavor, GitBranch, SkipGitSync, SkipBuild, SkipServices. The gateway MCP `create_env` tool (localhost:9877 inside a container) exposes these options if you need them from within an env.

### List envs

```bash
squire env
```

### SSH into an env

```bash
squire ssh <id> -- "cd /workspace && git status"
```

Non-interactive command execution. Quotes required around the remote command.

### Attach to the agent TUI

```bash
squire attach <id>
```

Opens OpenCode's TUI. Use this to watch the agent work or intervene.

## Non-Default Repo Pattern

The default image ships with only the c1 repo. For other repos, the workflow is:

1. **Create the env** (without a prompt):
   ```bash
   squire new my-feature --no-open
   ```

2. **Clone via git bundle** (NOT via `git clone` URL):
   The env's git credential helper only has access to repos the squire
   GitHub App is installed on (currently: c1). For other repos, create
   a local bundle and SCP it in:
   ```bash
   # On your laptop — bundle the branch you need
   git -C ~/repo/other-repo bundle create /tmp/repo.bundle branch-name

   # Transfer to the env
   scp /tmp/repo.bundle <env-name>.squire:/tmp/repo.bundle

   # Clone from the bundle inside the env
   squire ssh <id> -- "git clone /tmp/repo.bundle /data/squire/src/other-repo"
   squire ssh <id> -- "git -C /data/squire/src/other-repo checkout branch-name"
   ```

   Do NOT use `git clone git@github.com:...` — the credential helper
   will fail with `could not read Username` for repos not covered by
   the GitHub App. The only fix for that is an org-admin installing the
   squire GitHub App on the repo (GitHub org settings > Installations >
   configure > add repo). If you have admin access, do that instead of
   bundling.

3. **Send the prompt via OpenCode API** (correct format matters):
   ```bash
   squire ssh <id> -- 'SID=$(curl -sf -X POST http://localhost:4096/session | jq -r ".id") && sleep 2 && curl -sf -X POST http://localhost:4096/session/$SID/prompt_async -H "Content-Type: application/json" -d "{\"messageID\":\"msg_001\",\"parts\":[{\"type\":\"text\",\"text\":\"Your prompt here\"}],\"model\":{\"providerID\":\"anthropic\",\"modelID\":\"claude-opus-4-6\"}}"'
   ```

   The default model configured in the env is `anthropic/claude-opus-4-6`.

## OpenCode API Protocol

The in-env agent is OpenCode (NOT Claude Code). Sending prompts requires
a three-step API flow on `http://localhost:4096`:

1. `POST /session` — create a session, returns `{"id": "ses_..."}`.
2. Wait 2s for the session to initialize.
3. `POST /session/{id}/prompt_async` — fire-and-forget prompt delivery.

**Critical: the payload format is NOT `{"content": "..."}`.** It is:

```json
{
  "messageID": "msg_unique_id",
  "parts": [{"type": "text", "text": "Your prompt text"}],
  "model": {"providerID": "anthropic", "modelID": "claude-opus-4-6"}
}
```

- `messageID` must be unique per prompt (used for dedup on retry).
- `parts` is an array of message parts (text, images, etc.).
- `model` must match an available provider+model. The default image
  has `anthropic/claude-opus-4-6` configured. Using a wrong model ID
  (e.g. `claude-sonnet-4-20250514`) produces a silent
  `ProviderModelNotFoundError` — the session shows 0 messages and the
  agent never starts. Check `/home/squire/.local/share/opencode/log/`
  for the actual error.

## Monitoring Agent Progress

```bash
# Check session status
squire ssh <id> -- 'curl -sf http://localhost:4096/session/<ses_id> | jq "{title, directory}"'

# Check if agent is actively streaming
squire ssh <id> -- "tail -5 /home/squire/.local/share/opencode/log/*.log"

# Check for code changes
squire ssh <id> -- "git -C /data/squire/src/repo diff --stat"
squire ssh <id> -- "git -C /data/squire/src/repo log --oneline -5"

# Verify model hasn't drifted (should show claude-opus-4-6)
squire ssh <id> -- "grep modelID /home/squire/.local/share/opencode/log/*.log | tail -1"
```

## Model Enforcement

Approved models (any of these are acceptable):
- `anthropic/claude-opus-4-6` (default, preferred)
- `anthropic/claude-opus-4-7` or newer Claude Opus versions
- `openai/gpt-5.4`

Agents can drift to cheaper models mid-session (OpenCode's whitelist
also includes haiku, sonnet, GPT-5.4-mini/nano, and third-party
models). Cheaper models produce lower-quality Occult axioms and subtle
bugs.

**Always include the `model` field in every `prompt_async` call.** The
model field in the payload overrides the session default — this is the
primary enforcement mechanism.

**Check the whitelist before dispatching.** The env's whitelist may not
include the model you want. A model ID not in the whitelist produces a
silent `ProviderModelNotFoundError` — the session shows 0 messages and
the agent never starts. Before first dispatch to a new env, run:
```bash
squire ssh <id> -- "cat /home/squire/.config/opencode/opencode.json | jq '.provider.anthropic.whitelist'"
```
Use the newest Opus in the whitelist. As of 2026-04-17, newer env
images ship with `claude-opus-4-7` only (not 4-6).

**Verify on every polling tick** (add to your monitoring checks):

```bash
# Check what model the config says (default for new sessions)
squire ssh <id> -- "cat /home/squire/.config/opencode/opencode.json | jq '.model'"
# Expected: "anthropic/claude-opus-4-6" or newer Opus

# Check the active session's model (null = uses config default)
squire ssh <id> -- 'curl -sf http://localhost:4096/session | jq ".[-1].model"'

# Check recent log for model switches (look for different modelID)
squire ssh <id> -- "grep modelID /home/squire/.local/share/opencode/log/*.log | tail -3"
```

**If you detect model drift** to a non-approved model (haiku, sonnet,
mini, nano, or third-party): send a new prompt via `prompt_async` with
an approved model in the model field. The model field in `prompt_async`
is authoritative for that prompt — it overrides whatever the session
was using.

**Do NOT rely on the config file alone.** The config sets the default,
but OpenCode can switch models per-prompt. The `prompt_async` model
field is the only guarantee.

## Background Agent Polling

When delegating work to Squire envs, set up a `/loop` to periodically
check agent status so you're notified when work completes or stalls
without manually polling. This frees you to continue design work with
the user.

```
/loop 270s Check all running Squire envs: for each, SSH in and check
git log for new commits vs the base SHA you sent them. Report which
envs have committed, which are still working (uncommitted diff), and
which appear stalled (no changes and no recent log activity). Only
report if there's a state change since last check.
```

Use 270s (under the 5-minute cache TTL) to keep context warm. The loop
auto-notifies you when agents finish or stall — no need to manually
check between design discussions with the user.

**When to use:** Any time you've fired off one or more Squire envs and
want to continue other work. Kill the loop once all envs have completed
or been extracted.

**What to report:** Only state changes. "env X committed and pushed"
or "env Y appears stalled — no log activity for 10 minutes" are
useful. "env X still running" every 5 minutes is noise.

### Delegation to cheaper models

Some polling operations are purely mechanical and safe to delegate to
Haiku subagents. This saves context in the main conversation.

**Safe for Haiku (model: "haiku"):**
- **Batch polling.** A single Haiku subagent SSHes into all N envs,
  collects git log, git status, and the latest log timestamp, then
  returns a structured summary. This collapses N×3 tool calls into
  one Agent call. Example prompt:

  ```
  Check these squire envs for git state. For each, SSH in and
  collect: (1) git log --oneline -3, (2) git status --short,
  (3) tail -1 of the opencode log. Return a structured summary
  with env name, latest commit SHA, file count, and last log
  timestamp. Do NOT modify any files or run any commands besides
  these reads.

  Envs:
  - stormy-moose-21769
  - noble-cobra-20045
  ```

- **Bundle extraction.** SSH, bundle create, SCP, fetch, create
  review branch, report stats. Already proven reliable with Haiku.
- **Env setup.** SCP bundle, git clone, checkout branch.

**Keep in Opus (requires judgment):**
- **Stall detection.** Interpreting timestamps, process lists, and
  compaction events to decide "nudge" vs "wait" vs "cut off."
- **Cherry-pick + conflict resolution.** Understanding which changes
  are additive vs semantic requires reading the code.
- **Dispatch prompt authoring.** Task prompts need doc-reading
  instructions, correct model IDs, quality gates, and project
  context.
- **Task selection.** Choosing what to dispatch based on dependencies,
  file overlap risk, and difficulty.

**Pattern:** Haiku collects data, Opus interprets and decides. The
Haiku subagent returns raw facts; Opus applies the stall/nudge/cut-off
rules and picks next actions. This keeps the main context lean while
preserving decision quality.

## Dispatch Backlog and Autonomous Queue

When you have multiple tasks that are mechanical and well-scoped,
maintain a dispatch backlog and use the polling loop to drain it
autonomously. The loop checks running envs AND dispatches the next
queued task when a slot opens.

### Backlog format

Keep a mental (or written) ordered list of tasks:

```
BACKLOG:
1. [RUNNING: brave-panther-44637] Fix lint + reserved + min_len
2. [QUEUED] Phase 6: int versioning, collapse policy_ref
3. [QUEUED] Phase 4: device trust lane comments
4. [BLOCKED on #1] Runtime-cut rebase + field renames
5. [HUMAN] Sharding audit — needs design decision on partition strategy
```

### Dispatch rules

1. **Max 2 concurrent Squire envs** for the same branch — avoids
   push conflicts. One active + one queued is the sweet spot.
2. **On dispatch: `bd update <id> --claim`** to mark the issue
   in_progress. This keeps the bd task list accurate. Without this,
   `bd list --status=in_progress` shows stale data.
3. **Dispatch the next QUEUED item when the running env finishes**
   (commits + pushes). The polling loop handles this automatically.
4. **BLOCKED items wait** until their dependency finishes and pushes.
   The loop checks the dependency, and when satisfied, promotes the
   blocked item to QUEUED.
5. **HUMAN items stop the queue.** Report the decision needed to the
   user and wait. Don't dispatch past a HUMAN item — later items may
   depend on the decision.
6. **After each push, wait 5 min then check PR for feedback** (per
   github-pr-threads skill). New bot findings may add items to the
   backlog.

### bd lifecycle for squire tasks

```
bd create         → open (backlog)
bd update --claim → in_progress (dispatched to squire env)
bd close          → closed (merged + pushed)
```

Every dispatched task MUST go through in_progress. If you dispatch
without claiming first, `bd list --status=in_progress` is wrong and
the user can't see what's actually running. Claim at dispatch time,
not after.

### Estimated wall clock per task

Based on observed Squire behavior:
- Small (comment-only, validation tweak): ~15 min
- Medium (proto edits + worldgen + build): ~45 min (includes stall + recovery)
- Large (multi-file rename + worldgen + lint): ~60 min

Use these estimates to set expectations with the user and to decide
whether to bundle small tasks into one env.

### Loop integration

The polling cron doubles as the dispatch loop:

```
/loop 270s
For each running Squire env: check git log for commits.
If an env committed+pushed:
  - Pull locally, check PR for new feedback
  - Reply+resolve addressed threads
  - Dispatch next QUEUED item from backlog
  - Report to user
If an env is stalled: send finish prompt.
If backlog is empty and no envs running: kill the loop.
```

### Drain mode

When the user asks to pause, wind down, or take a break: enter drain
mode. In drain mode the loop continues polling but only merges — it
does NOT dispatch new tasks to idle envs.

**Entering drain mode (manual):**
- The user says "take a break", "pause dispatches", "drain", etc.
- Acknowledge and switch: "Drain mode — merging commits, no new dispatches."

**Entering drain mode (automatic):**
- A rate limit error occurs on any API call (429, "rate limited",
  "waiting for capacity"). This means the session is near its usage
  ceiling — stop creating new work and finish what's in flight.
- Context has compacted 3+ times in a single polling session. Heavy
  compaction indicates the session is long-running and approaching
  limits.
- In both cases: announce "Auto-drain — rate limit / session limit
  approaching. Merging remaining commits, no new dispatches." The
  user can override with "keep dispatching" if they want to push
  through.

**Note:** Claude Code has no programmatic quota API. These triggers
are heuristic, not authoritative. The user can always check remaining
capacity with `/usage` in the interactive prompt.

**In drain mode:**
- Continue polling all running envs at the same cadence.
- When an env commits: extract, merge, close the bd issue, push.
- Do NOT claim or dispatch new tasks to that env.
- Nudge stalled envs once to commit.
- **Cut-off rule:** If an env has not committed within one polling
  tick after being nudged, abandon it. Update the bd issue with a
  note about what was attempted, leave the issue open (not closed),
  and move on. Drain means drain — don't wait indefinitely for
  struggling agents.
- When all envs have committed, been cut off, or stopped: report
  to the user and kill the polling loop.
- **Update the cron prompt** when entering drain mode. The loop
  prompt must not encourage task dispatch. Replace it with a
  drain-specific prompt that only checks for commits and merges.

**Exiting drain mode:**
- The user says "resume", "dispatch more", "back to work", etc.
- Re-enter normal dispatch mode.

### When NOT to use the queue

- Tasks requiring design decisions (mark HUMAN)
- Tasks that touch the same files as a running env (push conflict)
- Tasks where the prompt requires judgment the agent can't make
  (e.g., "decide whether to keep or remove this feature")
- Tasks where failure is expensive (destructive git operations,
  production deployments)

## Extracting Work from Envs

Envs without GitHub App access for the repo can't `git push`. Use git
bundle to extract commits. Delegate to a subagent for parallel extraction.

```bash
# On the env: bundle only new commits (beyond shared base)
squire ssh <id> -- "git -C /data/squire/src/repo bundle create /tmp/work.bundle <base-sha>..HEAD"

# Transfer to local
scp <env-name>.squire:/tmp/work.bundle /tmp/<env-name>-work.bundle

# Fetch into local repo
git -C /path/to/repo fetch /tmp/<env-name>-work.bundle

# Create review branch
git -C /path/to/repo branch <review-branch> FETCH_HEAD

# Inspect
git -C /path/to/repo show --stat <review-branch>
```

When extracting from multiple envs, delegate to a subagent with all env
details in a single prompt. The subagent runs the bundle/scp/fetch/branch
steps for each env in parallel. This keeps the main conversation focused
on design work while extraction happens in the background.

**Merge notes:** If multiple envs branched from the same base and modified
overlapping files (e.g. both touched a test file), merging their review
branches into the feature branch will require conflict resolution.

**Post-merge checklist:**
1. Close the bd issue with the merge commit SHA.
2. If the bd issue corresponds to a numbered TODO.md item, mark it done
   in `docs/operations/TODO.md` with strikethrough + completion note.
   Example: `~~#519. LSP lint warnings~~ DONE -- wired in commit abc123.`
3. Push the branch.
4. Update the env with a fresh bundle before dispatching the next task.

## Quality Gates

Quality gates are project-specific. Define a project's gate bundle ONCE in a project-specific skill (e.g. `c1-squire-dispatch`) or in the project's `.claude/CLAUDE.md`. The squire brief invokes the bundle by name — "run the standard gate bundle before declaring success" — without enumerating gates per dispatch. The remote agent expands the bundle to the relevant subset based on what changed.

A dispatch is not done until every applicable gate is green. If a gate fails, fix the underlying issue — never skip.

Generic minimum (when no project-specific bundle is defined): "Run the project's full test suite and build before committing. Do NOT commit if anything fails."

## Brief Templates Per Task Family

Squire dispatches benefit from a project-specific table of task families. Each row defines:
- The skills the remote agent should load
- The env shape (minimal vs full project stack)
- Standing always-actives that apply on top

Define the table in a project-specific skill (e.g. `c1-squire-dispatch`). The dispatching session picks the matching row and pastes it into the brief. This stops re-deriving the manifest every dispatch and turns scattered briefs into a system.

When no row matches the task, the work is not yet ready for squire dispatch. Either decompose it or extend the table first.

Build the table from *actual* dispatch briefs you've written, not from speculation. Real dispatches reveal the rows; the table catalogs what already worked.

## Beads Dispatch Manifest

When a project uses beads (bd) for issue tracking, encode the dispatch brief in the bead so the agent picking it up doesn't re-derive context. Append this block to the bead description:

```
## Squire Dispatch
- Family: <project-defined>
- Task: <one of the project's task-family rows>
- Skills: standard | custom: <comma-separated overrides>
- Env: <project-defined env shapes>
- Gates: standard | custom: <list>
```

Rules:
1. `Skills: standard` and `Gates: standard` resolve against the project's task-family table and gate bundle.
2. The manifest is the source of truth — paste verbatim into the squire brief; resolve `standard` against the project skill.
3. If no row matches the task, the bead is not dispatch-ready. Decompose or extend the table.

This makes the bead self-briefing. The agent picking it up does not re-derive the manifest.

## Failure Debrief Protocol

When a returned squire dispatch is poor, do NOT immediately redesign the brief. The hypothesis you'd form is contaminated by your guess at what went wrong; the agent's actual experience is the data you need.

Trigger conditions (any of):
- Returned diff doesn't compile or fails the project's gate bundle.
- Off-target: wrong files touched, scope ignored, explicit constraints violated.
- Tests added are pass-through, shaped wrong, or assert at the wrong layer.
- Reasoning shows the agent misread the task despite a clear brief.

Protocol:
1. **First failure of this shape** — run `peace-agent-interview` on the returned agent. Get the uncontaminated account of what it understood, tried, and observed. Do not redesign the brief yet.
2. **Two or more failures of the same shape** — run `abc-agent-management` over the PEACE outputs. Identify antecedent (signal in the brief), behavior (what the agent did), consequence (what went wrong). Redesign the brief.
3. **Update the project's task-family table** if the failure reveals a missing skill, wrong env, or systematic gap. The table improves monotonically — every failure that changed the brief should leave a fingerprint in the table.

Never skip step 1. PEACE before ABC; clean data before analysis. Skipping PEACE produces brief redesigns based on your hypothesis of what went wrong, not the agent's actual experience.

## Parallel Envs

For independent work items, create multiple envs:

```bash
squire new auth-fix --prompt "Fix token refresh bug in pkg/auth"
squire new api-perf --prompt "Profile and optimize the sync endpoint"
squire new logging --prompt "Add structured logging to connector lifecycle"
```

Each env is fully isolated. Monitor all of them:

```bash
squire env                        # list all running envs
squire attach <id>                # check on a specific env
squire ssh <id> -- "git log -3"   # check progress without attaching
```

## Envmgr MCP Tools (localhost:9877)

Available inside the container for service and environment management:

| Tool | Purpose |
|------|---------|
| `env_status` | Current env state |
| `env_reload` | Reload env with new workspace_path or config |
| `list_services` | Show running services |
| `build_service` | Build a specific service |
| `start_service` | Start a service |
| `stop_service` | Stop a service |
| `restart_service` | Restart a service |
| `service_logs` | Tail service logs |
| `get_endpoints` | Show exposed endpoints |
| `env_self_update` | Update the env manager itself |

## Gateway MCP (inside env only)

The gateway MCP `create_env` tool is accessible from within an env and exposes additional options not in the CLI: `model`, `flavor`, `git_branch`. Use this for env-to-env delegation when one agent needs to spin up sub-environments.

## Common Mistakes

- **Using `git clone` URL for repos not in the GitHub App** — the env's credential helper only covers repos the squire GitHub App is installed on. For other repos, use `git bundle` + `scp` (see above). The symptom is `could not read Username for 'https://github.com/...'`.
- **Wrong model ID in prompt_async** — using a model string like `claude-sonnet-4-20250514` silently fails with `ProviderModelNotFoundError`. The session shows 0 messages and the agent never starts. The correct default is `claude-opus-4-6`. Check the env's config: `cat /home/squire/.config/opencode/opencode.json | jq '.model'`.
- **Wrong prompt payload format** — `{"content": "..."}` does NOT work. Must use `{"messageID": "...", "parts": [{"type": "text", "text": "..."}]}`. See OpenCode API Protocol above.
- **Piping binary data through `squire ssh`** — SSH via squire mangles binary. The pipe `cat file | squire ssh <id> -- "cat > dest"` corrupts non-UTF-8 content. Use `scp <file> <env>.squire:/path` instead.
- **Forgetting `--no-open`** — without it, `squire new` opens the TUI immediately. Use `--no-open` for scripted/parallel workflows.
- **Missing quotes around SSH commands** — `squire ssh <id> -- cd /foo && bar` runs `bar` locally. Always quote: `squire ssh <id> -- "cd /foo && bar"`.
- **Stale bd lock from crashed subagents** — if a subagent held the beads DB lock and crashed, `bd` fails with "another process holds the exclusive lock". Fix: `rm /path/to/.beads/embeddeddolt/.lock` (verify no process holds it with `lsof` first).
- **Workspace directory** — OpenCode's working directory defaults to `/data/squire/src` (containing c1). Your prompt must tell the agent the full path to the repo (e.g. `/data/squire/src/occult`). The `directory` field in the session object shows where the agent is actually working.
- **Model drift mid-session** — OpenCode's whitelist includes cheaper models (haiku, sonnet, GPT variants). The agent can switch models mid-session without warning, producing lower-quality output. Always include the `model` field in every `prompt_async` call and verify the model on polling ticks by grepping `modelID` in the logs. See "Model Enforcement" section above.
