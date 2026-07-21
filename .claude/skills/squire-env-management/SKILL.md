---
name: squire-env-management
description: >-
  Create and manage Squire ephemeral development environments for parallel agent
  work. Use when delegating implementation tasks to remote environments, creating
  fire-and-forget work sessions, or monitoring parallel agents. Triggers on:
  squire, ephemeral env, parallel agents, fire-and-forget, remote development.
---

# Squire Env Management

> **Status: partially deprecated.** Multi-env / batch orchestration (parallel
> envs, background polling, dispatch backlog/queue) is superseded by the
> `sqfan` skill: declarative `batch.yaml`, typed event stream, evidence on
> every transition, paranoid-doubt API. Sections marked **Legacy** below are
> reference-only (sqfan workarounds, single-env diagnostics). Canonical here:
> Core Commands (single-env), OpenCode API Protocol, Monitoring, Extracting
> Work, Multi-Repo / Non-Default Repo patterns, Envmgr MCP, Gateway MCP.
> Quality Gates, Brief Templates, Beads Manifest, Failure Debrief, Completion
> Metrics, Model Enforcement, and Common Mistakes are also folded into
> `sqfan` (which references this skill back for single-env work).

Squire provisions ephemeral dev environments — per-env container, services,
and agent session. The in-env agent is OpenCode, NOT Claude Code.

## Core Commands

```bash
squire new <name> --no-attach --model opus --prompt "Implement feature X in pkg/foo"
```

`--prompt` is delivered once the env is ready. For a single-repo task whose
repo is in the image (or clonable in-env, e.g. c1) this suffices — no manual
OpenCode API dance needed.

Flags (verified 2026-07-14; the set moves — check `squire new --help`):
- `--no-attach` — create without attaching to the TUI (older CLIs: `--no-open`).
- `--attach` — attach to the TUI after sending the prompt.
- `-m, --model` — `opus` | `sonnet` | `haiku` | full model ID. Pass `opus` for an approved model.
- `-p, --prompt` — initial fire-and-forget prompt.
- `-f, --flavor` — env size: `xxsmall|xsmall|small|medium|large|xlarge` (default `small`).
- `--git-branch` — branch to check out on startup.
- `--image` — image ID/slug (overrides `default_image`).
- `--skip-sync` / `--skip-build` / `--skip-services` — skip git reset / build_cmd / service startup at boot.
- `--timeout` — wait for env-ready (default `5m0s`).
- `--codex-effort` — reasoning effort, codex harness only.

The gateway MCP `create_env` tool at localhost:9877 exposes the same options
from within an env.

```bash
squire env                                          # list envs
squire ssh <id> -- "cd /workspace && git status"    # non-interactive; quote the remote command
squire attach <id>                                  # OpenCode TUI (watch or intervene)
```

## Exposed Service URLs

```text
https://<service-or-hostname-prefix>--<env-id>.<region>.squire.ductone.com/<path>
https://website--crystal-bee-83212.us-west-2.squire.ductone.com/pricing
```

Nested hostnames repeat `--`: a C1 tenant host `c1dev.<installation-domain>`
behind an exposed `envoy` service becomes
`https://c1dev--envoy--crystal-bee-83212.us-west-2.squire.ductone.com/`.

The `expose` section in `.squire/squire.yaml` lists what is public. For
unexposed services use `squire tunnel -e <env-id> -p <remote-port>` or
`squire tunnel -e <env-id> -s <service>`.

## C1 Runtime Note

In a running C1 env, do not use Tilt (that is for local Kubernetes and image
builds); the runtime is `squire-envmgr` with `.squire/squire.yaml`.
Fixture/auth bootstrap in-env: `dev-util ensure`, `dev-util ensure-tenant`,
`dev-util mint-test-client`. Long-lived services started over `squire ssh`
must be detached with `setsid -f`, or they receive SIGTERM when the SSH
command exits.

## Multi-Repo Dispatches (Latchkey)

Most Latchkey work spans repos. Enumerate every repo the task plausibly
touches and bundle all of them up front (bundles are typically <1 MB); do not
let the agent discover a missing dependency at compile time.

| Repo | Path in env | What lives here |
|---|---|---|
| `latchkey-proto` | `/data/squire/src/latchkey-proto` | Canonical proto schemas for V4 API + models + service contracts. |
| `latchkey-mls-core` | `/data/squire/src/latchkey-mls-core` | MLS adapter + OpenMLS shim. Owns `MlsGroupRuntime`, `CommitReceipt`, `IncomingEvent`, `StreamKind`. |
| `latchkey-client-sdk` | `/data/squire/src/latchkey-client-sdk` | Rust SDK consumed by every native client. Re-exports the relevant mls-core types. Wraps the c1 gRPC stubs. |
| `latchkey-client-shells` | `/data/squire/src/latchkey-client-shells` | Top-level Rust workspace: the `latchkey` CLI binary at the root + non-CLI shell scaffolds under `shells/`. |
| `latchkey-desktop` | `/data/squire/src/latchkey-desktop` | Standalone Tauri 2.x desktop client (React/Vite + Rust). |
| `c1` | `/data/squire/src/c1` | The C1 monorepo. Frontend lives under `frontend/`; backend Go under `pkg/`. |

Cross-repo shapes: "add a CLI command" almost always touches the SDK (new
`LatchkeyClient`/`VaultStore` method) and may touch the proto; "Tauri
Keychain support" touches desktop + SDK (`keychain` module,
`MacosKeychainStateStore`); a proto field-type change touches proto + SDK +
every consumer of the field.

Dispatch steps:

1. Clone each bundle to its canonical path (`/data/squire/src/<repo-name>`),
   check out the branch (typically `rch/feature/latchkey-api-v4-*`).
2. Patch each Rust consumer's `Cargo.toml` with a
   `[patch."ssh://git@github.com/ductone/<repo>.git"]` block pointing at the
   sibling working tree — env-only build convenience; the agent must NOT
   commit this patch table (see Common Mistakes).
3. In the prompt, name the sibling repos explicitly: a sibling change (new
   SDK type, proto field) goes in `/data/squire/src/<repo>` on its working
   branch, committed there as well — never papered over with a `[patch]`.
4. Final status note lists branch + commit SHA range per sibling repo
   touched; extraction bundles each non-c1 repo back separately.

## Non-Default Repo Pattern

Newer images may launch with `/data/squire/src/` empty (older ones pre-cloned
c1). Verify: `squire ssh <id> -- "ls /data/squire/src/"`. The env's git
credential helper covers ductone/c1, so
`git clone https://github.com/ductone/c1.git /data/squire/src/c1` works
directly (~30-60 s); do it during env prep, not at the agent's first compile.

Other repos — the credential helper only covers repos the squire GitHub App
is installed on (currently: c1). Use a git bundle:

```bash
# On your laptop — bundle the branch you need
git -C ~/repo/other-repo bundle create /tmp/repo.bundle branch-name

# Transfer to the env
scp /tmp/repo.bundle <env-name>.squire:/tmp/repo.bundle

# Clone from the bundle inside the env
squire ssh <id> -- "git clone /tmp/repo.bundle /data/squire/src/other-repo"
squire ssh <id> -- "git -C /data/squire/src/other-repo checkout branch-name"
```

Do NOT `git clone git@github.com:...` — it fails with
`could not read Username`. The only alternative is an org admin installing
the squire GitHub App on the repo (org settings > Installations > configure >
add repo).

Then send the prompt via the OpenCode API (below).

## OpenCode API Protocol

**The env's authenticated OpenCode listens on a RANDOM port, not 4096.** The
image launches `opencode-linux-arm64/bin/opencode serve` at env-start with
`ANTHROPIC_API_KEY` in its process environment, on a randomly-assigned port:

```bash
squire ssh <env> -- 'ss -tlnp | grep "opencode" || pgrep -af "opencode.*serve"'
```

Flow (`$PORT` = discovered port):

1. `POST http://localhost:$PORT/session` with body `{}` — returns `{"id": "ses_..."}`.
2. Wait 2 s for the session to initialize.
3. `POST http://localhost:$PORT/session/{id}/prompt_async` — returns `204 No Content` on accept.

Payload — all four fields required:

```json
{
  "messageID": "msg_unique_id",
  "role": "user",
  "parts": [{"type": "text", "text": "Your prompt text"}],
  "model": {"providerID": "anthropic", "modelID": "claude-opus-4-7"}
}
```

- Missing `role: "user"` fails silently: server returns 204, agent errors
  with `"No user message found in stream. This should never happen."`, model
  never invoked.
- `messageID` must be unique per prompt (dedup on retry).
- A wrong model ID (e.g. `claude-sonnet-4-20250514`) produces a silent
  `ProviderModelNotFoundError` — session shows 0 messages. Errors land in
  `/home/squire/.local/share/opencode/log/`.

Build the payload JSON locally with the Write tool and `scp` it in — in-env
`jq` edits can produce JSON that passes shape validation but is rejected at
model-invocation time:

```bash
scp /tmp/my-prompt.json <env>.squire:/tmp/prompt.json
squire ssh <env> -- "curl -sf -X POST http://localhost:$PORT/session/$SID/prompt_async \
  -H 'Content-Type: application/json' --data @/tmp/prompt.json -w 'http=%{http_code}\n'"
```

### Recovering from `ProviderAuthError`

A self-started `opencode serve` (e.g. on 4096) inherits no API key; every
prompt fails with `ProviderAuthError: Anthropic API key is missing`. Extract
the key from the authenticated process and relaunch:

```bash
squire ssh <env> -- '
  PID=$(pgrep -f "linux-arm64/bin/opencode" | head -1)
  KEY=$(tr "\0" "\n" < /proc/$PID/environ | grep "^ANTHROPIC_API_KEY=" | cut -d= -f2-)
  pkill -f "opencode serve --port 4096" 2>/dev/null || true
  sleep 2
  ANTHROPIC_API_KEY="$KEY" nohup opencode serve --port 4096 --hostname 127.0.0.1 \
    >/tmp/opencode.log 2>&1 &
'
```

### The `question` tool deadlocks the agent

If OpenCode invokes its built-in `question` tool the agent halts until
answered. `POST /session/{sid}/question/{qid}` returns 200 but routes to the
web UI SPA — there is no programmatic answer API.

- Every dispatch prompt must include: *"HARD RULE: do not use the `question`
  tool under any circumstance. If you encounter a decision point, pick the
  most reasonable default, document the choice in your status note, and
  continue."*
- Stalled with no work worth saving: create a fresh session and re-dispatch
  with the rule embedded.
- Stalled with commits/edits worth saving, and the question is
  human-answerable in seconds: answer via the OpenCode UI proxy at
  `https://opencode--<env-id>.us-west-2.squire.ductone.com/` (env-id from
  `squire env`; authenticate via squire SSO; the pending question renders in
  the UI and the agent resumes). Not for fire-and-forget fleets — the ban
  stays the default there.

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

Approved models:
- `anthropic/claude-opus-4-6` (default, preferred)
- `anthropic/claude-opus-4-7` or newer Claude Opus
- `openai/gpt-5.4`

OpenCode's whitelist also includes haiku, sonnet, GPT-5.4-mini/nano, and
third-party models; agents can drift to them mid-session, degrading output
(lower-quality Occult axioms, subtle bugs).

- Include the `model` field in EVERY `prompt_async` call. It overrides the
  session default and is the only guarantee — the config file only sets the
  default for new sessions, and OpenCode can switch models per-prompt.
- Check the whitelist before first dispatch to a new env; a model ID not in
  it fails silently (`ProviderModelNotFoundError`, 0 messages):

  ```bash
  squire ssh <id> -- "cat /home/squire/.config/opencode/opencode.json | jq '.provider.anthropic.whitelist'"
  ```

  Use the newest Opus present. As of 2026-04-17, newer env images ship
  `claude-opus-4-7` only (not 4-6).
- Verify on every polling tick:

  ```bash
  # Config default for new sessions — expect "anthropic/claude-opus-4-6" or newer Opus
  squire ssh <id> -- "cat /home/squire/.config/opencode/opencode.json | jq '.model'"

  # Active session's model (null = config default)
  squire ssh <id> -- 'curl -sf http://localhost:4096/session | jq ".[-1].model"'

  # Recent log for model switches
  squire ssh <id> -- "grep modelID /home/squire/.local/share/opencode/log/*.log | tail -3"
  ```
- On drift to a non-approved model: send a new `prompt_async` with an
  approved model in the model field — it is authoritative for that prompt.

## Legacy: Background Agent Polling (superseded by sqfan)

> sqfan's `poll` MCP tool (blocks until the next typed event, carries
> evidence) replaces `/loop` polling for any dispatch producing >1 env. Kept
> for sqfan workarounds and single-env diagnostics.

```
/loop 270s Check all running Squire envs: for each, SSH in and check git log
for new commits vs the base SHA you sent them. Report which envs have
committed, which are still working (uncommitted diff), and which appear
stalled (no changes and no recent log activity). Only report state changes.
```

270 s stays under the 5-minute cache TTL. Report only state changes —
"committed and pushed" or "stalled, no log activity for 10 minutes"; not
"still running". Kill the loop once all envs are completed or extracted.

### Delegation to cheaper models

Safe for Haiku: batch polling (one subagent SSHes all N envs, returns git
log/status + last log timestamp as a structured summary — collapses N×3 tool
calls into one), bundle extraction (SSH, bundle, scp, fetch, review branch),
env setup (scp bundle, clone, checkout). Always include "Do NOT modify any
files" in the Haiku prompt.

Keep in Opus: stall detection (nudge vs wait vs cut off), cherry-pick +
conflict resolution, dispatch prompt authoring, task selection. Haiku
collects raw facts; Opus interprets and decides.

## Legacy: Dispatch Backlog and Autonomous Queue (superseded by sqfan)

> Replaced by sqfan's declarative `batch.yaml`. Kept for reference.

Dispatch rules:

1. Max 2 concurrent envs on the same branch (push conflicts); one active +
   one queued is the sweet spot.
2. `bd update <id> --claim` at dispatch time, not after — otherwise
   `bd list --status=in_progress` is stale. Lifecycle: `bd create` → open;
   `--claim` → in_progress (dispatched); `bd close` → closed (merged + pushed).
3. Dispatch the next QUEUED item when a running env commits + pushes.
4. BLOCKED items wait until their dependency finishes and pushes.
5. HUMAN items stop the queue — report the decision needed and wait; later
   items may depend on it.
6. After each push, wait 5 min then check the PR for feedback
   (github-pr-threads skill).

Observed wall clock: small (comment-only, validation tweak) ~15 min; medium
(proto edits + worldgen + build) ~45 min; large (multi-file rename +
worldgen + lint) ~60 min.

Loop integration — the polling loop doubles as dispatcher. On commit+push:
pull locally, check PR feedback, reply+resolve addressed threads, record
metrics (`~/repo/dotfiles/scripts/squire-metrics.sh record <env-id>`),
dispatch next QUEUED item, report. Stalled env: send finish prompt. Empty
backlog + no envs: kill the loop. `squire env info <env-id>` exposes
`Created:`, which the metrics script parses — no side-table needed.

### Drain mode

Poll and merge only; no new dispatches. Enter on: user request ("pause",
"drain", "take a break"); any rate-limit error (429, "rate limited",
"waiting for capacity"); 3+ compactions in one polling session. Announce the
switch; user can override with "keep dispatching". These triggers are
heuristic — no programmatic quota API exists; the user can check `/usage`.

In drain: on commit — extract, merge, close the bd issue, push. Nudge stalled
envs once; cut off any env that hasn't committed within one polling tick
after the nudge (note the attempt on the bd issue, leave it open). Update the
cron prompt to a drain-only prompt (no dispatch language). Exit on "resume".

### When NOT to use the queue

- Tasks requiring design decisions (mark HUMAN)
- Tasks touching the same files as a running env (push conflict)
- Prompts requiring judgment the agent can't make
- Expensive failure modes (destructive git ops, production deploys)

## Legacy: Parallel Envs (superseded by sqfan)

Multi-env work belongs in a sqfan `batch.yaml`, not a sequence of bare
`squire new` calls.

## Extracting Work from Envs

Envs without GitHub App access for the repo can't `git push`; extract via git
bundle. For multiple envs, delegate all bundle/scp/fetch/branch steps to one
subagent with every env's details in a single prompt.

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

Envs branched from the same base that modified overlapping files will
conflict when merging their review branches.

Post-merge: (1) close the bd issue with the merge commit SHA; (2) if it maps
to a numbered TODO.md item, strike it through in `docs/operations/TODO.md`
with a completion note (e.g. `~~#519. LSP lint warnings~~ DONE -- wired in
commit abc123.`); (3) push the branch; (4) refresh the env with a new bundle
before the next dispatch.

## Quality Gates

Define a project's gate bundle ONCE in a project-specific skill (e.g.
`c1-squire-dispatch`) or the project's `.claude/CLAUDE.md`; the brief invokes
it by name — "run the standard gate bundle before declaring success" — and
the remote agent expands it to the relevant subset. A dispatch is not done
until every applicable gate is green; fix failures, never skip. Generic
minimum when no bundle exists: "Run the project's full test suite and build
before committing. Do NOT commit if anything fails."

## Brief Templates Per Task Family

Keep a project-specific task-family table — per row: skills the remote agent
loads, env shape (minimal vs full stack), standing always-actives — in a
project skill (e.g. `c1-squire-dispatch`). The dispatching session pastes the
matching row into the brief. No matching row = the work is not
dispatch-ready: decompose it or extend the table. Build rows from actual
dispatch briefs, not speculation.

## Beads Dispatch Manifest

For bd-tracked projects, append to the bead description so the bead is
self-briefing:

```
## Squire Dispatch
- Family: <project-defined>
- Task: <one of the project's task-family rows>
- Skills: standard | custom: <comma-separated overrides>
- Env: <project-defined env shapes>
- Gates: standard | custom: <list>
```

`standard` resolves against the project's task-family table and gate bundle.
The manifest is the source of truth — paste it verbatim into the squire
brief. No matching row = the bead is not dispatch-ready.

## Failure Debrief Protocol

On a poor dispatch (diff fails gates or doesn't compile; off-target scope or
violated constraints; pass-through / wrong-layer tests; agent misread a clear
brief), do NOT immediately redesign the brief:

1. First failure of a shape: run `peace-agent-interview` on the returned
   agent for its uncontaminated account.
2. Two+ failures of the same shape: run `abc-agent-management` over the PEACE
   outputs (antecedent/behavior/consequence), then redesign the brief.
3. Update the project's task-family table if the failure reveals a missing
   skill, wrong env, or systematic gap.

PEACE before ABC — never skip step 1.

## Completion Metrics

One JSONL line per completed dispatch, captured at extraction time. Fields:
`env_id`, `env_name`, `started_at` (auto-pulled from `squire env info`'s
`Created:` field), `completed_at`, `duration_seconds`, `branch`, `base`
(defaults to `merge-base origin/main <branch>`; when a dispatch EXTENDS a
branch with prior commits, pass `--base <head-before-dispatch>` or the
LOC/file counts include the prior work), `commit_count`, `files_changed`,
`lines_added`, `lines_removed`.

File: `~/repo/dotfiles/scripts/squire-metrics.jsonl` (override via
`SQUIRE_METRICS_FILE`).

```bash
squire-metrics.sh record <env-id>    # SSHes into the env, computes diff stats
squire-metrics.sh tally [--last N]   # min/median/mean/max duration + LOC totals
```

Record at branch-push completion or after bundle extraction. Do NOT record
stalled, cut-off, or no-commit failed envs. Refine the task-family table's
wall-clock estimates once N >= 5 records exist per family.

## Envmgr MCP Tools (localhost:9877)

Available inside the container:

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

The gateway MCP `create_env` tool (options: `model`, `flavor`, `git_branch`)
enables env-to-env delegation. Same options now exist on `squire new`; use
the MCP form only from within an env.

## Common Mistakes

- **`git clone` URL for repos not in the GitHub App** — the credential helper
  only covers App-installed repos; symptom:
  `could not read Username for 'https://github.com/...'`. Use bundle + scp.
- **Assuming `gh` is authenticated** — `gh` IS installed (2026-06 images:
  v2.93.0) but logged into no host, and no token exists in the env's
  environment. `gh pr diff` / `gh pr view` / `gh api` fail with
  `You are not logged into any GitHub hosts` (silent if stderr unchecked).
  Two patterns: (1) pre-stage via scp — run
  `gh pr diff <num> --repo <owner>/<repo> > /tmp/pr-<num>.diff` (and
  `gh pr view --json ...`) locally, scp into the env; the agent reads the
  files. Cheapest for bounded one-shot tasks like code review. (2) Mint a
  token via the envmgr `git_token` MCP tool at localhost:9877 (JSON-RPC
  protocol in `c1-dev-stack-in-squire`): a short-lived (~30 min; long tasks
  re-mint) GitHub App installation token scoped to App repos (today: just
  `ductone/c1`); the dispatch prompt includes the minting recipe and exports
  `GH_TOKEN` before gh calls. Choose (2) only for many API calls.
- **Wrong model ID in prompt_async** — silent `ProviderModelNotFoundError`,
  session shows 0 messages. Check
  `cat /home/squire/.config/opencode/opencode.json | jq '.model'`.
- **Wrong payload format** — `{"content": "..."}` does NOT work; the payload
  requires `messageID`, `role: "user"`, `parts`, `model`. Missing
  `role: "user"` is the silent killer (204, "No user message found in
  stream", model never invoked).
- **Talking to port 4096 instead of the env's random port** — 4096 only
  listens if you started your own server, which lacks the API key; prompts
  fail with `ProviderAuthError`. Discover the real port:
  `ss -tlnp | grep opencode` or `pgrep -af "opencode-linux-arm64.*serve"`.
- **Editing prompt JSON in-env with `jq`** — passes shape checks, rejected at
  model invocation. Write locally, scp in.
- **`question` tool stall** — no programmatic answer API. Ban it in every
  dispatch prompt; recover via the UI proxy or re-dispatch (see above).
- **Stopped env not resumable from old CLI** — envs auto-stop on idle; wake
  with `squire env start <name>`. Older CLIs lack `start` (error: "accepts at
  most 1 arg(s), received 2"); upgrade: `make -C ~/repo/squire install`. Disk
  state, including commits, survives stop. After restart, opencode is on a
  possibly-different random port; any self-started 4096 server must be
  relaunched with the API key.
- **Polling loop probing dead envs** — add "if env status=stopped, report and
  stop polling that env" to the polling prompt.
- **Sibling-repo types shipped one-sided (Latchkey)** — the CLI lives in
  `latchkey-client-shells`, the SDK in `latchkey-client-sdk`, the proto in
  `latchkey-proto`, the MLS adapter in `latchkey-mls-core`, the desktop in
  `latchkey-desktop`. An agent that needs a sibling change and instead
  commits a `[patch]` table pointing at `/data/squire/src/` working trees
  papers over the gap in-env; on a clean clone the consumer fails to compile.
  A committed `[patch]` table resolving to sibling working trees always
  represents a missing upstream change. Brief explicitly: either (a) make the
  sibling change in the same dispatch, committing to both repos' branches, or
  (b) stop at the boundary and document the required sibling change in the
  status note. Name the canonical repo roots (`/data/squire/src/<repo>`) in
  the prompt.
- **Piping binary data through `squire ssh`** — mangles non-UTF-8 content
  (`cat file | squire ssh <id> -- "cat > dest"` corrupts). Use
  `scp <file> <env>.squire:/path`.
- **Forgetting `--no-attach`** — `squire new` attaches to the TUI. Current
  builds reject the old `--no-open` name with "unknown flag".
- **Missing quotes around SSH commands** —
  `squire ssh <id> -- cd /foo && bar` runs `bar` locally. Always quote.
- **Stale bd lock from crashed subagents** — `bd` fails with "another process
  holds the exclusive lock". Fix: `rm /path/to/.beads/embeddeddolt/.lock`
  (verify no process holds it with `lsof` first).
- **Workspace directory** — OpenCode's cwd defaults to `/data/squire/src`;
  newer images may launch with it empty. Verify with `ls`, clone repos to
  canonical paths before prompting, and name the full repo path in the prompt
  (e.g. `/data/squire/src/c1`). The session object's `directory` field shows
  where the agent is actually working.
- **OpenCode log files may not exist** — newer images run
  `opencode serve --print-logs` with no file rotation, so
  `/home/squire/.local/share/opencode/log/` is absent and `grep ...log/*.log`
  errors. Verify model via the session API instead:
  `curl -sf http://localhost:<port>/session/<sid>/message | jq '.[-1].metadata.assistant.modelID'`
  (discover the port first).
- **Model drift mid-session** — see Model Enforcement: `model` field in every
  `prompt_async`, verify on polling ticks.
- **A single invalid MCP tool schema silently kills every model call** —
  OpenCode wires every configured MCP server's tools into each Anthropic
  request; any tool whose input_schema uses top-level `oneOf`/`allOf`/`anyOf`
  gets the whole request rejected (400 invalid_request_error, e.g.
  `tools.168.custom.input_schema: input_schema does not support oneOf, allOf,
  or anyOf at the top level`). Symptom: `prompt_async` returns 204, user
  messages store, a session title may appear, but zero assistant output and
  zero file changes. Triage: send "Reply with exactly: READY" to a fresh
  session, then read the stored error:
  `curl -s localhost:$PORT/session/$SID/message | jq '[.[] | select(.info.role=="assistant")][-1].info.error'`.
  Fix: disable MCP servers in `/home/squire/.config/opencode/opencode.json`
  (`jq '.mcp |= with_entries(.value.enabled = false)'`), restart the serve
  process with the API key. Key recovery when the old process is gone: scan
  `pgrep -f squire-envmgr` pids for a readable `/proc/<pid>/environ`
  containing `ANTHROPIC_API_KEY` (pid 1 is envmgr but unreadable; a child
  like the static-server carries it). Most fire-and-forget dispatches need no
  env-side MCP — disable MCP proactively at dispatch time.
