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
and agent session. Primary harness is OMP (Oh My Pi), not Claude Code.
Default model is DeepSeek V4 Flash; Astra is the strong pin. The CLI
surface moves; when a flag matters, verify with `--help` before scripting it.


## Common Mistakes

- **Sibling task on the dispatching env** — MUST NOT. Default spawn is a
  child env (`squire new` / squine `env_create`), one env per item.
  Sibling `squire task create` is only for a batch on a dedicated shared
  env created for that batch, never the laptop session and never a
  live-pass inner env.
- **Spawn without a bead** — MUST NOT. `bd create`, then `--claim`, then
  spawn. No bead, no spawn. Bead ids stay internal (never in PRs or
  commits).
- **Treating notify_parent as laptop poll** — `notify_parent` wakes an
  in-Squire calling task. From the laptop it is a no-op. Laptop wait is
  squine `poll` / `github_poll`. Pass it anyway (squine defaults true;
  CLI `--notify-parent`).
- **Passing a long brief as a shell argument** — backticks and `$(...)` in
  the brief get shell-interpreted. Use `squire task create --prompt-file`
  (or scp a brief file in and send a one-line pointer prompt).
- **Assuming `-p` attaches** — `squire new -p` now exits immediately
  (fire-and-forget by default); `--attach` to watch. Older docs said the
  opposite.
- **Cloning with SSH URLs, or treating `gh auth status` as git auth** —
  Squire rewrites `git@github.com:` and `ssh://git@github.com/` to HTTPS
  and mints GitHub App tokens via `git-credential-squire`. Clone
  `https://github.com/org/repo.git`. `gh auth status` can say logged out
  while that helper still works. `could not read Username for
  'https://github.com'` has two causes: the helper did not run (common
  on `squire ssh -- git clone` with no TTY), or the App does not cover
  the repo. Ask the in-env agent to clone over HTTPS. If the helper
  returns nothing, install the App. Do not git-bundle inject or extract.
- **`--skip-sync` on an image that does not bake the repo** — skips
  fetch/reset; `base-default` stays empty. Unset already follows
  flavor/image. Do not pass `--skip-sync` as a clone workaround.
- **Assuming `gh` is not logged in** — the env `gh` wrapper and git
  credential helper use the credential socket. `git push` and `gh pr create`
  are the return path. PRs are stamped as opened from Squire.
- **Short model aliases** — `-m opus` resolves to a retired pin and 400s
  (`unknown or disabled model "anthropic/claude-opus-4-8"`, seen 2026-08-07).
  Pass a full ID; the 400's `detail` lists what the gateway accepts.
- **Wrong prompt payload** — `{"content": "..."}` does not work; missing
  `role: "user"` is the silent killer. See the Fallback section.
- **Talking to port 4096** — the authenticated server is on a random port;
  a self-started 4096 server lacks the API key.
- **Editing payload JSON in-env with `jq`** — passes shape checks, fails at
  model invocation. Write locally, scp in.
- **Blocking `question` on fire-and-forget** — OpenCode question is
  Squire-answerable (`POST .../interactive-input/{requestID}/answer`)
  only if a boss is listening (attach, web/desktop, or a poller).
  Claude `AskUserQuestion` is display_only. No listener = stall.
  Prefer `squire.message.send`; else default and document.
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
squire new <name> -p "Fix the login bug" --harness omp -m together/deepseek-ai/DeepSeek-V4-Flash-0731
squire new <name> -p "Hard refactor" --harness omp -m openai/gpt-6-astra
squire new <name> --no-attach
squire env
squire ssh <id> -- "cd /workspace && git status"
squire attach <id>
```

`squire new` with `-p` is fire-and-forget BY DEFAULT: the prompt is sent when
the env is ready and the CLI exits immediately. `--attach` overrides to watch.
Useful flags (re-verify, the set moves): `-m/--model`, `--harness`,
`-f/--flavor` (`xxsmall`…`xlarge`), `--image`, `--git-branch`,
`--skip-sync`/`--skip-build`/`--skip-services`, `--timeout`.

**Pass `-m` a full model ID, never a short alias.** `-m opus` resolves to a
retired pin and 400s. Together/DeepSeek without `--harness omp` falls through
to OpenCode (provider `*` canonical harness), not OMP.

Default / daily: `together/deepseek-ai/DeepSeek-V4-Flash-0731`
Mid: `together/deepseek-ai/DeepSeek-V4-Pro-0813`
Strong: `openai/gpt-6-astra`

All three are OMP-compatible. Laptop CLI defaults (`squire config list`):
`new.harness` / `task.create.harness` = `omp`, model = Flash. Explicit
`--harness` / `-m` still win. Web UI and MCP env_create do not read those
CLI defaults.

## Dispatch (MUST)

1. Bead first: `bd create`, `--claim`, then spawn. No bead, no spawn.
2. Child env per item: `squire new` / squine `env_create` with the brief
   (`--prompt-file` when long). MUST NOT `squire task create` on the
   dispatching env.
3. Batch exception: first item `squire new` as a dedicated shared env;
   the rest `squire task create --notify-parent` on that shared env with
   isolated worktrees. Never the inner/live-pass env.
4. `notify_parent` on spawn. Squire no-ops from laptop; laptop wait is
   squine `poll`. In-Squire parent wakes.

## Agent Tasks (batch / follow-up on a dedicated shared env)

`squire task` drives task lifecycle over gateway HTTP — prefer it to the raw
OpenCode API for follow-up and for the batch exception above:

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
  mangling. Write the brief under `plans/<topic>/` next to the RFC/plan,
  then `--prompt-file` it (or `-` for stdin).
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

## Multi-Repo Dispatches

Most of that work spans repos. Enumerate the plausible set up front and
put ALL of them in the env; do not let the agent discover a missing
dependency at compile time.

Repos go under a source root you pick — `/data/squire/src/` is the
convention below. A repo the IMAGE ships is wherever the image put it
(c1 was at `/data/src/c1` on 2026-08-07), so resolve those per Non-Default
Repo Pattern rather than reading a path off this table.

| Repo | What lives here |
|---|---|
| `latchkey-proto` | Canonical proto schemas for V4 API + models + service contracts. |
| `latchkey-mls-core` | MLS adapter + OpenMLS shim. |
| `latchkey-client-sdk` | Rust SDK consumed by every native client. |
| `multipass-cli` | CLI binary + shell scaffolds. |
| `multipass-desktop` | Tauri 2.x desktop client. |
| `c1` | The C1 monorepo (image-provided; resolve the path). |

"Add a CLI command" almost always touches the SDK and may touch the proto.
Include extra repos speculatively. Clone each to its canonical path on the
right branch if the GitHub App covers it. Patch Rust consumers' `Cargo.toml`
with `[patch]` blocks pointing at sibling trees for the env build ONLY (the
agent must NOT commit that patch table — see Common Mistakes). Name the
sibling repos in the brief. Require per-repo branch + SHA-range in the
status note; return path is `gh pr create` per repo.

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

If the checkout is genuinely absent, ask the in-env agent to
`git clone https://github.com/org/repo.git` (credential helper, HTTPS).
Do not clone over SSH. Do not clone via `squire ssh -- git clone` as
the first try. If the helper returns no credentials, install the App
on that repo. Do not git-bundle inject.

## Monitoring Agent Progress

Drive work with in-env agent prompts (`squire task create` / `squire attach -p`). `squire ssh` is locate/probe only:

```bash
squire task list --env <env>                       # states, newest first
squire diff out -e <env> | head -50                # what changed so far
squire ssh <id> -- "git -C /data/squire/src/<repo> log --oneline -5"
squire ssh <id> -- "git -C /data/squire/src/<repo> diff --stat"
```

## Model Enforcement

Cheaper models produce lower-quality output and subtle bugs; agents can
drift mid-session. Discipline:

- Pin harness and model at creation (`squire new --harness omp -m <full-id>`)
  and on `squire task create`. Full IDs only. Short aliases are retired pins
  (see Core Commands).
- Approved default: `together/deepseek-ai/DeepSeek-V4-Flash-0731`. Strong:
  `openai/gpt-6-astra`. A model ID the gateway does not list fails at spawn
  (`squire models` is the catalog).
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
  envs (task list + git log + git status), env setup.
  Brief MUST say do NOT modify any files.
- **Main model (judgment):** stall detection (nudge vs wait vs cut off),
  PR tracking, brief authoring, task selection.
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
2. Bead first: `bd create`, then `bd update <id> --claim`, then spawn.
   `bd list --status=in_progress` must reflect reality. No bead, no spawn.
3. Dispatch the next QUEUED item when a running env opens a PR (or pushes).
4. BLOCKED items wait for their dependency's push; then promote to QUEUED.
5. HUMAN items stop the queue — report the decision and wait; later items
   may depend on it.
6. After each push, check the PR for feedback (github-pr-threads skill);
   new findings append to the backlog.

bd lifecycle: `bd create` before spawn → open; `--claim` then spawn →
in_progress; `bd close` after merge+push. Wall-clock priors (refine from
metrics): small ~15 min, medium ~45 min, large ~60 min.

### Drain mode

Enter on: user asks to pause/wind down; a rate-limit error; heavy repeated
compaction. Announce it. In drain: keep polling, track PRs that land, dispatch
NOTHING new, nudge stalled envs once, cut off any env that doesn't push a PR
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
squire new auth-fix -p "Fix token refresh bug in pkg/auth" --harness omp -m together/deepseek-ai/DeepSeek-V4-Flash-0731
squire new api-perf -p "Profile and optimize the sync endpoint" --harness omp -m openai/gpt-6-astra
```

Each is isolated. `squire env` lists; `squire attach --mux` opens one pane
per active task across all running envs.

## Return path

Finished work is an in-env `git push` and `gh pr create` (credential socket
and `gh` wrapper). Track that PR. Review with `squire diff out`. Optional
local landing without GitHub: `squire git remote add`. Do not git-bundle
harvest.

If push fails with `could not read Username` after a helper-backed
HTTPS remote, the GitHub App is not installed on that repo. Install
the App. That is not a bundle protocol. SSH remotes are rewritten to
HTTPS; do not "fix" this by switching to SSH.

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

One JSONL line per completed dispatch at PR-open or branch-push time:
`~/repo/dotfiles/scripts/squire-metrics.sh record <env-id>` (pulls
`started_at` from `squire env info`); `squire-metrics.sh tally [--last N]`
aggregates. Fields: env id/name, started/completed timestamps, duration,
branch, base SHA, commit count, files/LOC. Record harness, model,
isolation, verifier, and PR/CI outcome on the same line when known.
That JSONL is the feed for a laptop observation (squine cell later).
Record on PR open or push;
do NOT record stalled/cut-off/failed envs. For dispatches extending an
existing branch, pass `--base <head-before-dispatch>` or the LOC counts
absorb prior work. Refine the task-family wall-clock estimates once N >= 5
per family.

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

### Asking the boss

OpenCode native question is `answerable` on Squire interactive-input when
a client is watching. Claude `AskUserQuestion` is `display_only`. Headless
fire-and-forget has no listener: a blocking question stalls.

Prefer `squire.message.send` to the parent (non-blocking). Use native
question only with a boss in the loop (attach, web/desktop, or a poller
on `interactive_input`). Fire-and-forget briefs: pick a default, write it
in the status note, continue. Recovery for an already-stalled OpenCode
session: answer via the interactive-input RPC or the OpenCode UI proxy at
`https://opencode--<env-id>.us-west-2.squire.ductone.com/`.

## Envmgr MCP Tools (localhost:9877, in-env)

`env_status`, `env_reload`, `list_services`, `build_service`,
`start_service`, `stop_service`, `restart_service`, `service_logs`,
`get_endpoints`, `env_self_update`. The gateway MCP `create_env` (in-env
only) enables env-to-env delegation; the same options are on `squire new`
for CLI callers.

## Before finishing

- [ ] Bead claimed, child env per item, `notify_parent` (CLI `--notify-parent`; squine defaults true)?
- [ ] Brief via `--prompt-file` (not shell-arg) if long?
- [ ] Full model ID; fire-and-forget brief has no blocking question?
- [ ] Gates defined and green before claiming done?
- [ ] No committed env-only `[patch]` tables?
