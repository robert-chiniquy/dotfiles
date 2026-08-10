---
name: squire-env-management
description: >-
  Create and manage Squire ephemeral development environments for parallel agent
  work. Use when delegating implementation tasks to remote environments, creating
  fire-and-forget work sessions, monitoring parallel agents, or scheduling
  recurring agent runs. Triggers on: squire, ephemeral env, parallel agents,
  fire-and-forget, remote development, agent task, arena, squire workflow.
---

# Squire Env Management

> sqfan (a batch orchestrator formerly layered on these protocols) is
> **deprecated — never use it.** Everything below is squire-native. The
> dispatch-backlog, polling, and parallel-env protocols here are canonical
> again.

Squire provisions ephemeral dev environments — per-env container, services,
and agent session. The in-env agent is OpenCode, NOT Claude Code. The CLI
surface moves; when a flag matters, verify with `--help` before scripting it.


## Common Mistakes

- **Passing a long brief as a shell argument** — backticks and `$(...)` in
  the brief get shell-interpreted. Use `squire task create --prompt-file`
  (or scp a brief file in and send a one-line pointer prompt).
- **Assuming `-p` attaches** — `squire new -p` now exits immediately
  (fire-and-forget by default); `--attach` to watch. Older docs said the
  opposite.
- **Using `git clone` URL for repos not in the GitHub App** — use bundle +
  scp. Symptom: `could not read Username for 'https://github.com/...'`.
- **Assuming `gh` is authenticated in-env** — installed but NOT logged in.
  Either pre-stage data via scp before dispatch (cheapest for bounded tasks
  like review), or mint a short-lived token via the envmgr `git_token` MCP
  tool and export `GH_TOKEN` (expires ~30 min; only covers App repos).
- **Short model aliases** — `-m opus` resolves to a retired pin and 400s
  (`unknown or disabled model "anthropic/claude-opus-4-8"`, seen 2026-08-07).
  Pass a full ID; the 400's `detail` lists what the gateway accepts.
- **Wrong prompt payload** — `{"content": "..."}` does not work; missing
  `role: "user"` is the silent killer. See the Fallback section.
- **Talking to port 4096** — the authenticated server is on a random port;
  a self-started 4096 server lacks the API key.
- **Editing payload JSON in-env with `jq`** — passes shape checks, fails at
  model invocation. Write locally, scp in.
- **The `question` tool** — see above; ban it in every brief.
- **Stopped envs** — envs auto-stop on idle; `squire env start <name>`
  wakes them (disk state and commits preserved; the OpenCode port changes).
  Add "if env stopped, report and stop polling it" to loop prompts.
- **Committed `[patch]` tables (Rust multi-repo)** — a `[patch]` block
  pointing at `/data/squire/src/` siblings papers over a missing upstream
  change; on a clean clone the consumer fails to compile. Make the sibling
  change in the same dispatch (commits in both repos) or stop at the
  boundary and document it in the status note.
- **Piping binary through `squire ssh`** — mangles non-UTF-8. Use `scp`.
- **Missing quotes around ssh commands** — `squire ssh <id> -- cd /foo &&
  bar` runs `bar` locally.
- **Stale bd lock from crashed subagents** — `bd` reports another process
  holds the lock; verify with `lsof`, then remove
  `.beads/embeddeddolt/.lock`.
- **Hardcoding a repo path in the brief** — layouts differ per image, and an
  empty `/data/squire/src/` does not mean the repo is missing (c1 lived at
  `/data/src/c1` on 2026-08-07). Locate the checkout before dispatch and name
  the resolved path; the session `directory` field shows where the agent
  actually landed. See Non-Default Repo Pattern.
- **OpenCode log files may not exist** on newer images (`--print-logs`
  only). For model-drift checks, hit the session API:
  `curl -sf http://localhost:<port>/session/<sid>/message | jq '.[-1].metadata.assistant.modelID'`.

## Core Commands

```bash
squire new <name> -p "Fix the login bug" -m anthropic/claude-opus-5   # dispatch
squire new <name> --no-attach                      # create env, no prompt yet
squire env                                         # list envs / select one
squire ssh <id> -- "cd /workspace && git status"   # non-interactive exec (quote it)
squire attach <id>                                 # watch the agent TUI (multiplayer)
```

`squire new` with `-p` is fire-and-forget BY DEFAULT: the prompt is sent when
the env is ready and the CLI exits immediately. `--attach` overrides to watch.
Useful flags (verified 2026-08-05; re-verify, the set moves): `-m/--model`,
`-f/--flavor` (`xxsmall`…`xlarge`), `--image`, `--git-branch`,
`--skip-sync`/`--skip-build`/`--skip-services`, `--timeout`.

**Pass `-m` a full model ID, never a short alias.** The aliases resolve to
retired pins: `-m opus` failed 2026-08-07 with `400 unknown or disabled model
"anthropic/claude-opus-4-8"`. The 400's `detail` field lists every id the
gateway currently accepts, so read the error rather than guessing a successor.

## Agent Tasks (primary dispatch + follow-up surface)

`squire task` drives task lifecycle over gateway HTTP — prefer it to the raw
OpenCode API for everything it covers:

```bash
squire task create --env <env> --prompt-file brief.md --title 'fix-auth'
squire task list --env <env>          # newest first
squire task get <task-id>
squire task resume <task-id>          # resume a terminal task in place
squire task complete <task-id>        # terminate with completed state + summary
squire task delete <task-id>          # reap worktree + branch refs + session
squire attach <task-id> -p "follow-up message"   # nudge a running task
```

- **Long briefs go through `--prompt-file`, never shell arguments.** Briefs
  contain backticks and `$(...)`; passing them as `-p` strings invites shell
  mangling. Write the brief with the Write tool, then `--prompt-file` it (or
  `-` for stdin).
- `squire attach <task-id>` resolves task IDs across all visible envs — no
  selected env needed. Multiple args or `--mux` opens one pane per active
  task in tmux/zellij.
- The task TUI is multiplayer with ring-buffer replay: attaching late still
  shows history, and multiple viewers share the PTY.

## Reviewing env work

```bash
squire diff out -e <env>                 # unified diff, everything-on-branch
squire diff out -e <env> --base <ref>    # explicit base
squire env info <env>                    # details incl. Created: timestamp
```

`squire diff out` covers the same file set as the web UI diff panel
(merge-base of origin/main by default). Pipe to `delta` for reading.

## Event-driven monitoring (arenas, messages, workflows)

- `squire arena watch` — SSE stream of live arena events; the event-driven
  alternative to timed polling when envs belong to an arena.
- `squire message send|claim|complete|list` — arena messages are the IPC
  primitive handoff/steering/kick flows ride on; usable by external
  orchestrators and humans, same tools in-env agents use.
- `squire workflow def|schedule|run` — cron-scheduled agent runs. Use for
  recurring dispatches instead of hand-rolled loops.
- `squire fs` — read/write an arena's shared filesystem (cross-env artifact
  handoff).

These postdate the older protocol below; prefer them when an arena exists.
For plain unaffiliated envs, the polling protocol further down still applies.

## Exposed Service URLs

Gateway-exposed services:

```text
https://<service-or-hostname-prefix>--<env-id>.<region>.squire.ductone.com/<path>
```

Nested hostnames repeat the `--` separator: tenant host `c1dev.<domain>`
behind an exposed `envoy` becomes
`https://c1dev--envoy--<env-id>.us-west-2.squire.ductone.com/`. The `expose`
section of `.squire/squire.yaml` says what is public; for anything else use
`squire tunnel -e <env-id> -p <port>` (or `-s <service>`).

## C1 Runtime Note

In a running C1 env, do not use Tilt — the env runtime is `squire-envmgr`
driven by `.squire/squire.yaml`. Fixture/auth bootstrap uses `dev-util
ensure`, `dev-util ensure-tenant`, and `dev-util mint-test-client` from
inside the env. Long-lived services started over `squire ssh` must be
detached with `setsid -f` or they die with the SSH session.

## Multi-Repo Dispatches (Latchkey)

Most Latchkey work spans repos. Enumerate the plausible set up front and
bundle ALL of them into the env; do not let the agent discover a missing
dependency at compile time.

Repos you bundle in go under a source root you pick — `/data/squire/src/` is
the convention below. A repo the IMAGE ships is wherever the image put it
(c1 was at `/data/src/c1` on 2026-08-07), so resolve those per Non-Default
Repo Pattern rather than reading a path off this table.

| Repo | What lives here |
|---|---|
| `latchkey-proto` | Canonical proto schemas for V4 API + models + service contracts. |
| `latchkey-mls-core` | MLS adapter + OpenMLS shim. |
| `latchkey-client-sdk` | Rust SDK consumed by every native client. |
| `latchkey-client-shells` | CLI binary + shell scaffolds. |
| `latchkey-desktop` | Tauri 2.x desktop client. |
| `c1` | The C1 monorepo (image-provided; resolve the path). |

"Add a CLI command" almost always touches the SDK and may touch the proto.
When bundling: bundle speculatively (bundles are small), clone each to its
canonical path on the right branch, patch Rust consumers' `Cargo.toml` with
`[patch]` blocks pointing at sibling trees for the env build ONLY (the agent
must NOT commit that patch table — see Common Mistakes), name the sibling
repos explicitly in the brief, and require per-repo branch + SHA-range
reporting in the status note so extraction can bundle each repo back.

## Non-Default Repo Pattern

**Find the checkout; never hardcode its path in a brief.** Image layouts
differ, and an empty `/data/squire/src/` does NOT mean the repo is absent —
on the 2026-08-07 c1 image that directory was empty while the checkout sat at
`/data/src/c1`. A brief that says "clone it if `/data/squire/src/c1` is empty"
sends the agent to duplicate a repo that already exists. Locate it first:

```bash
squire ssh <id> -- 'ls /data/squire/src/ /data/src/ 2>/dev/null'
squire ssh <id> -- 'find / -maxdepth 4 -type d -name <repo> -not -path "*/node_modules/*" 2>/dev/null'
```

Then name the resolved absolute path in the brief. Filter the `find` hits:
`*/cache/*` and `*/gocache/*` are build caches, not working trees.

c1 also clones directly if genuinely absent (credential helper covers it).
Other repos ship via git bundle:

```bash
git -C ~/repo/other-repo bundle create /tmp/repo.bundle branch-name
scp /tmp/repo.bundle <env-name>.squire:/tmp/repo.bundle
squire ssh <id> -- "git clone /tmp/repo.bundle /data/squire/src/other-repo"
squire ssh <id> -- "git -C /data/squire/src/other-repo checkout branch-name"
```

Do NOT `git clone git@github.com:...` for repos the squire GitHub App does
not cover — the credential helper fails with `could not read Username`. The
fix is installing the App on the repo (org admin), not fighting the helper.

## Monitoring Agent Progress

Prefer the task surface, fall back to ssh probes:

```bash
squire task list --env <env>                       # states, newest first
squire diff out -e <env> | head -50                # what changed so far
squire ssh <id> -- "git -C /data/squire/src/<repo> log --oneline -5"
squire ssh <id> -- "git -C /data/squire/src/<repo> diff --stat"
```

## Model Enforcement

Cheaper models produce lower-quality output and subtle bugs; agents can
drift mid-session. Discipline:

- Pin the model at creation (`squire new -m anthropic/claude-opus-5`) and in
  every raw `prompt_async` payload (the payload model field is authoritative
  per prompt). Full IDs only — short aliases are retired pins (see Core
  Commands).
- Approved: the newest Claude Opus available to the env (as of 2026-08:
  `anthropic/claude-opus-5`). Check the env's whitelist BEFORE first
  dispatch — a model ID not in the whitelist fails silently
  (`ProviderModelNotFoundError`, session shows 0 messages):
  `squire ssh <id> -- "cat /home/squire/.config/opencode/opencode.json | jq '.provider.anthropic.whitelist'"`
- Verify on polling ticks; on drift, send the next prompt with an approved
  model in the payload model field.

## Background Agent Polling

When envs are in flight, set up a `/loop` so completion/stalls surface
without manual polling:

```
/loop 270s Check all running Squire envs: for each, check squire task list
and git log vs the base SHA dispatched. Report ONLY state changes: which
envs committed, which are still working (uncommitted diff), which look
stalled (no changes, no recent activity).
```

Delegation split (keeps main context lean):
- **Cheap read-only subagent (mechanical reads):** batch polling across N
  envs (task list + git log + git status), bundle extraction, env setup.
  Brief MUST say do NOT modify any files.
- **Main model (judgment):** stall detection (nudge vs wait vs cut off),
  cherry-pick/conflict resolution, brief authoring, task selection.
  Pattern: cheap subagent collects facts; main model decides.

## Dispatch Backlog and Autonomous Queue

For multiple mechanical, well-scoped tasks, keep an ordered backlog and let
the polling loop drain it:

```
BACKLOG:
1. [RUNNING: brave-panther-44637] Fix lint + reserved + min_len
2. [QUEUED] Phase 6: int versioning
3. [BLOCKED on #1] Runtime-cut rebase
4. [HUMAN] Sharding audit — needs a design decision
```

Rules:
1. Max 2 concurrent envs on the same branch (push conflicts); one active +
   one queued is the sweet spot.
2. `bd update <id> --claim` AT dispatch time — `bd list --status=in_progress`
   must reflect reality.
3. Dispatch the next QUEUED item when a running env commits+pushes.
4. BLOCKED items wait for their dependency's push; then promote to QUEUED.
5. HUMAN items stop the queue — report the decision and wait; later items
   may depend on it.
6. After each push, check the PR for feedback (github-pr-threads skill);
   new findings append to the backlog.

bd lifecycle: `bd create` → open; `--claim` at dispatch → in_progress;
`bd close` after merge+push. Wall-clock priors (refine from metrics): small
~15 min, medium ~45 min, large ~60 min.

### Drain mode

Enter on: user asks to pause/wind down; a rate-limit error; heavy repeated
compaction. Announce it. In drain: keep polling, merge what commits, dispatch
NOTHING new, nudge stalled envs once, cut off any env that doesn't commit
within one tick of its nudge (note the bead, leave it open, move on). Exit on
user resume. Update the loop prompt when entering — the drain prompt must not
encourage dispatch.

### When NOT to queue

Design-decision tasks (mark HUMAN); tasks touching the same files as a
running env; tasks needing judgment the agent can't make; tasks where
failure is expensive (destructive git, deployments).

## Parallel Envs

Independent work items get independent envs:

```bash
squire new auth-fix -p "Fix token refresh bug in pkg/auth" -m anthropic/claude-opus-5
squire new api-perf -p "Profile and optimize the sync endpoint" -m anthropic/claude-opus-5
```

Each is isolated. `squire env` lists; `squire attach --mux` opens one pane
per active task across all running envs.

## Extracting Work from Envs

Envs without GitHub App access for the repo can't push. Extract via bundle
(cheap read-only subagent OK):

```bash
squire ssh <id> -- "git -C /data/squire/src/repo bundle create /tmp/work.bundle <base-sha>..HEAD"
scp <env-name>.squire:/tmp/work.bundle /tmp/<env-name>-work.bundle
git -C /path/to/repo fetch /tmp/<env-name>-work.bundle
git -C /path/to/repo branch <review-branch> FETCH_HEAD
```

Post-merge checklist: close the bd issue with the merge SHA; mark any
TODO.md item done; push the branch; refresh the env's bundle before its next
task. Overlapping-file merges across envs will need conflict resolution.

## Quality Gates

Gates are project-specific: define a project's bundle ONCE (project skill or
`.claude/CLAUDE.md`), and have briefs invoke it by name ("run the standard
gate bundle"). A dispatch is not done until every applicable gate is green;
fix, never skip. Generic minimum: full test suite + build before commit; do
not commit on red.

## Brief Templates Per Task Family

Define a per-project task-family table (skills to load, env shape,
always-actives) in the project's dispatch skill (`c1-squire-dispatch`,
`occult-squire-dispatch`). The dispatching session picks the row and pastes
it into the brief. No matching row = not dispatch-ready: decompose or extend
the table. Build rows from real dispatches, not speculation.

## Beads Dispatch Manifest

For bd-tracked projects, make the bead self-briefing:

```
## Squire Dispatch
- Family: <project-defined>
- Task: <one of the project's task-family rows>
- Skills: standard | custom: <comma-separated overrides>
- Env: <project-defined env shapes>
- Gates: standard | custom: <list>
```

`standard` resolves against the project skill. Paste the manifest verbatim
into the brief.

## Failure Debrief Protocol

When a dispatch comes back poor, do NOT immediately redesign the brief.
First failure of a shape: run `peace-agent-interview` on the returned agent
(uncontaminated account first). Two or more failures of the same shape: run
`abc-agent-management` over the PEACE outputs, then redesign. Every failure
that changes the brief should leave a fingerprint in the task-family table.
PEACE before ABC; clean data before analysis.

## Completion Metrics

One JSONL line per completed dispatch at extraction time:
`~/repo/dotfiles/scripts/squire-metrics.sh record <env-id>` (pulls
`started_at` from `squire env info`); `squire-metrics.sh tally [--last N]`
aggregates. Fields: env id/name, started/completed timestamps, duration,
branch, base SHA, commit count, files/LOC. Record on branch-push completion
or bundle extraction; do NOT record stalled/cut-off/failed envs. For
dispatches extending an existing branch, pass `--base <head-before-dispatch>`
or the LOC counts absorb prior work. Refine the task-family wall-clock
estimates once N >= 5 per family.

## Fallback: driving OpenCode directly

`squire task create` / `squire attach -p` wrap this; use the raw API only
for diagnostics or when the task surface fails.

The env's authenticated OpenCode listens on a RANDOM port (not 4096). Find
it: `squire ssh <env> -- 'ss -tlnp | grep opencode || pgrep -af "opencode.*serve"'`.
Protocol: `POST /session` (returns `{"id":"ses_..."}`), wait 2s,
`POST /session/{id}/prompt_async` (204 on accept). Payload requirements:

```json
{
  "messageID": "msg_unique_id",
  "role": "user",
  "parts": [{"type": "text", "text": "Your prompt text"}],
  "model": {"providerID": "anthropic", "modelID": "<newest whitelisted Opus>"}
}
```

- `role: "user"` is mandatory — without it the server 204s but the agent
  silently errors ("No user message found in stream") and never runs.
- `messageID` unique per prompt (dedup on retry).
- Wrong/unwhitelisted model ID = silent `ProviderModelNotFoundError`.
- Build payload JSON locally with the Write tool and `scp` it in; in-env
  `jq` edits can produce JSON that passes shape checks but fails at model
  invocation.
- If you start your own `opencode serve`, it inherits no API key
  (`ProviderAuthError`); extract `ANTHROPIC_API_KEY` from the existing
  process's `/proc/<pid>/environ` and relaunch with it.

### The `question` tool deadlocks the agent

There is no programmatic answer API. Every dispatch brief includes: "HARD
RULE: do not use the `question` tool under any circumstance. Pick the most
reasonable default, document the choice in your status note, continue."
Recovery for an already-stalled session: the OpenCode UI proxy at
`https://opencode--<env-id>.us-west-2.squire.ductone.com/` renders the
pending question — answer it there when the session holds work worth saving;
otherwise abandon and re-dispatch with the rule embedded.

## Envmgr MCP Tools (localhost:9877, in-env)

`env_status`, `env_reload`, `list_services`, `build_service`,
`start_service`, `stop_service`, `restart_service`, `service_logs`,
`get_endpoints`, `env_self_update`. The gateway MCP `create_env` (in-env
only) enables env-to-env delegation; the same options are on `squire new`
for CLI callers.

## Before finishing

- [ ] Brief via `--prompt-file` (not shell-arg) if long?
- [ ] Model pinned / whitelisted?
- [ ] `question` tool banned in brief?
- [ ] Gates defined and green before claiming done?
- [ ] No committed env-only `[patch]` tables?
