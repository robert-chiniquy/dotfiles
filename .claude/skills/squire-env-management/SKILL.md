---
name: squire-env-management
description: >-
  Create and manage Squire ephemeral development environments for parallel agent
  work. Use when delegating implementation tasks to remote environments, creating
  fire-and-forget work sessions, or monitoring parallel agents. Triggers on:
  squire, ephemeral env, parallel agents, fire-and-forget, remote development.
---

# Squire Env Management

> **Status: partially deprecated.** For multi-env / batch
> orchestration (Parallel Envs, Background Agent Polling, Dispatch
> Backlog & Autonomous Queue), prefer the `sqfan` skill. sqfan
> replaces the conversational backlog + `/loop` polling protocol with
> a declarative `batch.yaml`, a typed event stream, evidence on every
> transition, and a paranoid-doubt API. The sections marked
> **Legacy: superseded by sqfan** below remain for reference only —
> new dispatches should not use them.
>
> The following content STILL applies and is the canonical reference:
> Core Commands (single-env `squire new`), OpenCode API Protocol,
> Monitoring Agent Progress, Extracting Work from Envs, the Multi-
> Repo Dispatches / Non-Default Repo Pattern, Envmgr MCP Tools, and
> Gateway MCP. The accumulated domain knowledge — Quality Gates,
> Brief Templates, Beads Dispatch Manifest, Failure Debrief Protocol,
> Completion Metrics, Model Enforcement, and Common Mistakes — has
> been folded into the `sqfan` skill (which itself references this
> skill back for single-env work).

Squire provisions ephemeral dev environments with an in-env agent (OpenCode, not Claude Code). Use these for parallel, isolated work: each env gets its own container, services, and agent session.

## Core Commands

### Create an env (fire-and-forget)

```bash
squire new <name> --no-attach --model opus --prompt "Implement feature X in pkg/foo"
```

`--prompt` delivers an initial instruction fire-and-forget: it is sent once the env is ready, so for a single-repo task whose repo is in the image (or clonable in-env, e.g. c1) you do NOT need the manual OpenCode API dance below. `--no-attach` creates the env without opening its TUI (scripted/parallel). `--model` pins the agent model at creation.

Common `squire new` flags (verify with `squire new --help` — the set moves):
- `--no-attach` — create without attaching to the TUI. Older CLIs called this `--no-open`.
- `--attach` — attach to the TUI after sending the prompt.
- `-m, --model` — `opus` | `sonnet` | `haiku` | a full model ID. Pass `opus` to pin an approved model.
- `-p, --prompt` — initial fire-and-forget prompt.
- `-f, --flavor` — env size: `xxsmall|xsmall|small|medium|large|xlarge` (default `small`).
- `--git-branch` — branch to check out on startup.
- `--image` — image ID/slug (overrides `default_image`).
- `--skip-sync` / `--skip-build` / `--skip-services` — skip git reset / build_cmd / service startup at boot.
- `--timeout` — how long to wait for the env to be ready (default `5m0s`).
- `--codex-effort` — reasoning effort, codex harness only.

(Verified against the CLI 2026-07-14. `--model`/`--flavor`/`--git-branch`/`--skip-*` used to be API-only; they are now on the CLI. The gateway MCP `create_env` tool at localhost:9877 still exposes the same options from within an env.)

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

## Exposed Service URLs

For services exposed through the Squire gateway, public URLs generally use:

```text
https://<service-or-hostname-prefix>--<env-id>.<region>.squire.ductone.com/<path>
```

Example:

```text
https://website--crystal-bee-83212.us-west-2.squire.ductone.com/pricing
```

Nested hostnames use repeated `--` segments. For example, a C1 tenant host
`c1dev.<installation-domain>` behind an exposed `envoy` service becomes:

```text
https://c1dev--envoy--crystal-bee-83212.us-west-2.squire.ductone.com/
```

Use the `expose` section in `.squire/squire.yaml` to identify which service
names or hostname prefixes are public. If a service is not exposed through the
gateway, use `squire tunnel -e <env-id> -p <remote-port>` or
`squire tunnel -e <env-id> -s <service>` and point clients at the local tunnel.

## C1 Runtime Note

For an already-running C1 Squire env, do not use Tilt to start services. C1 docs
mention Tilt for local Kubernetes development and for building/registering the
Squire image, but the remote env runtime is `squire-envmgr` with
`.squire/squire.yaml`. C1 fixture/auth bootstrap uses `dev-util ensure`,
`dev-util ensure-tenant`, and `dev-util mint-test-client` from inside the env.
If manually starting long-lived services over `squire ssh`, detach them from the
SSH session with `setsid -f`; a plain background process may receive SIGTERM
when the SSH command exits.

## Multi-Repo Dispatches (Latchkey)

Most Latchkey work spans more than one repo. Before dispatching,
enumerate which of these repos the task is likely to touch and bundle
**all** of them into the env up front; do not wait for the agent to
discover a missing dependency at compile time.

| Repo | Path in env | What lives here |
|---|---|---|
| `latchkey-proto` | `/data/squire/src/latchkey-proto` | Canonical proto schemas for V4 API + models + service contracts. |
| `latchkey-mls-core` | `/data/squire/src/latchkey-mls-core` | MLS adapter + OpenMLS shim. Owns `MlsGroupRuntime`, `CommitReceipt`, `IncomingEvent`, `StreamKind`. |
| `latchkey-client-sdk` | `/data/squire/src/latchkey-client-sdk` | Rust SDK consumed by every native client. Re-exports the relevant mls-core types. Wraps the c1 gRPC stubs. |
| `latchkey-client-shells` | `/data/squire/src/latchkey-client-shells` | Top-level Rust workspace: the `latchkey` CLI binary at the root + non-CLI shell scaffolds under `shells/`. |
| `latchkey-desktop` | `/data/squire/src/latchkey-desktop` | Standalone Tauri 2.x desktop client (React/Vite + Rust). |
| `c1` | `/data/squire/src/c1` | The C1 monorepo. Frontend lives under `frontend/`; backend Go under `pkg/`. |

A task framed as "add a CLI command" almost always touches the SDK
(new method on `LatchkeyClient` or `VaultStore`) and may touch the
proto (new RPC field or message). A task framed as "Tauri Keychain
support" touches both the desktop repo and the SDK (`keychain` module
with `MacosKeychainStateStore`). A task framed as "the API now uses
bytes for SHA-256" touches the proto (field type), the SDK (struct
field name + type), and **every** consumer that references the field.

When bundling for the dispatch:

1. Bundle every repo the task plausibly touches, not just the
   "primary" one. The bundles are small (typically <1 MB each);
   bundling speculatively costs almost nothing and prevents the
   agent from getting stuck.
2. Inside the env, clone each bundle to its canonical path
   (`/data/squire/src/<repo-name>`) and check out the right branch
   (typically `rch/feature/latchkey-api-v4-*`).
3. Patch each Rust consumer's `Cargo.toml` with a
   `[patch."ssh://git@github.com/ductone/<repo>.git"]` block pointing
   at the sibling working tree. This is the convenience-patch that
   lets the in-env build resolve sibling crates without network
   access — and it is **for the env only**. The agent must **not**
   commit this patch table. See the "Common Mistakes" entry on
   committed `[patch]` tables for the failure mode.
4. In the dispatch prompt, name the sibling repos explicitly: "if
   you need to add a type to the SDK to make the CLI compile, do it
   in `/data/squire/src/latchkey-client-sdk` on its working branch;
   commit there as well as in the CLI repo; do not paper over the
   missing type with a `[patch]` table that points at the sibling
   tree."
5. Brief the agent on how to report multi-repo work: the final
   status note lists, per sibling repo touched, the branch and the
   commit SHA range. The extraction step then bundles each
   non-c1 repo separately back to the laptop.

## Non-Default Repo Pattern

The default image historically shipped with c1 pre-cloned at `/data/squire/src/c1`, but newer images may launch with `/data/squire/src/` empty. **Always verify before assuming**: `squire ssh <id> -- "ls /data/squire/src/"`. If c1 isn't there, clone it inline — the env's git credential helper covers ductone/c1, so `git clone https://github.com/ductone/c1.git /data/squire/src/c1` works directly (no bundle needed). The clone takes ~30-60 s; build it into the env-prep step rather than letting the agent discover the missing tree at first compile.

For other repos, the workflow is:

1. **Create the env** (without a prompt):
   ```bash
   squire new my-feature --no-attach
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
a three-step API flow over HTTP on the env's authenticated OpenCode
server.

**The env's authenticated OpenCode listens on a RANDOM port, not 4096.**
The default image launches `opencode-linux-arm64/bin/opencode serve` at
env-start with `ANTHROPIC_API_KEY` in its process environment, on a
randomly-assigned port. Find it with:

```bash
squire ssh <env> -- 'ss -tlnp | grep "opencode" || pgrep -af "opencode.*serve"'
```

Use the discovered port — for the rest of this protocol, treat it as
`$PORT`.

1. `POST http://localhost:$PORT/session` with body `{}` — create a
   session, returns `{"id": "ses_..."}`.
2. Wait 2s for the session to initialize.
3. `POST http://localhost:$PORT/session/{id}/prompt_async` — fire-and-
   forget prompt delivery. Returns `204 No Content` on accept.

**Critical: the payload format requires `role: "user"`.** Without it,
the server returns `204` but the agent silently errors with
`"No user message found in stream. This should never happen."` and
never invokes the model. The correct payload:

```json
{
  "messageID": "msg_unique_id",
  "role": "user",
  "parts": [{"type": "text", "text": "Your prompt text"}],
  "model": {"providerID": "anthropic", "modelID": "claude-opus-4-7"}
}
```

- `messageID` must be unique per prompt (used for dedup on retry).
- `role` must be `"user"`. Older skill docs omitted this and the
  agent silently failed; current OpenCode versions require it.
- `parts` is an array of message parts (text, images, etc.).
- `model` must match an available provider+model. Using a wrong model
  ID (e.g. `claude-sonnet-4-20250514`) produces a silent
  `ProviderModelNotFoundError` — the session shows 0 messages and the
  agent never starts. Check `/home/squire/.local/share/opencode/log/`
  for the actual error.

**Build the JSON payload locally with the Write tool and `scp` it
in.** In-env `jq` edits of payload files can produce JSON that passes
shape validation but is rejected at model-invocation time (likely a
text-encoding issue). The reliable pattern:

```bash
# Local: write the payload file with Write
# Local: scp it in
scp /tmp/my-prompt.json <env>.squire:/tmp/prompt.json
# In env: POST the file
squire ssh <env> -- "curl -sf -X POST http://localhost:$PORT/session/$SID/prompt_async \
  -H 'Content-Type: application/json' --data @/tmp/prompt.json -w 'http=%{http_code}\n'"
```

### Recovering from `ProviderAuthError`

If you start your own `opencode serve` (e.g., on port 4096 for
convenience), it inherits no API key and every prompt fails with
`ProviderAuthError: Anthropic API key is missing`. Extract the key
from the env's existing authenticated process and relaunch:

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

If OpenCode invokes its built-in `question` tool, the agent halts
indefinitely until the question is answered. The HTTP endpoint
`POST /session/{sid}/question/{qid}` returns `200 OK` but routes to
the web UI SPA — **there is no programmatic answer API exposed by the
server**.

Mitigations:

- Include an explicit rule in the dispatch prompt: *"HARD RULE: do not
  use the `question` tool under any circumstance. If you encounter a
  decision point, pick the most reasonable default, document the
  choice in your status note, and continue."*
- If a session is already stalled on a question, the practical
  recovery is to create a fresh session and re-dispatch with the rule
  embedded. The old session can be abandoned.

**Recovery via the OpenCode UI proxy (unblock without abandoning):**

Every squire env exposes its OpenCode UI as a public proxy at:

```
https://opencode--<env-id>.us-west-2.squire.ductone.com/
```

where `<env-id>` is the env ID shown in `squire env` (e.g.
`giant-goat-55224` → `https://opencode--giant-goat-55224.us-west-2.squire.ductone.com/`).
This is a direct passthrough to the OpenCode web UI running in the
env, including the question-answer prompt. To unblock a stalled
session:

1. `squire env` to find the env-id of the stalled env.
2. Open `https://opencode--<env-id>.us-west-2.squire.ductone.com/` in
   a browser.
3. Authenticate (typical squire SSO flow).
4. The pending question renders in the UI — answer it there, the
   agent resumes from where it stalled.

When to use this instead of abandoning the session:

- The agent has done meaningful work (commits, edits) that you don't
  want to lose by re-dispatching.
- The question is one a human can answer in seconds (yes/no, file
  path choice, "is this the right approach").
- You're already at a keyboard. Don't use this for fire-and-forget
  fleets — the rule above (forbid the question tool entirely) is
  still the right default for autonomous dispatches.

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

## Legacy: Background Agent Polling (superseded by sqfan)

> **Deprecated.** sqfan's `poll` MCP tool replaces the `/loop`-driven
> polling described here. `poll` blocks until the next typed event
> fires (`status_change`, `committed`, `failure_fired`, etc.) and
> carries the evidence the gate consulted. Use sqfan for any
> dispatch that produces > 1 env. Sections kept below for reference
> when working around a sqfan limitation or doing single-env
> diagnostics.

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

## Legacy: Dispatch Backlog and Autonomous Queue (superseded by sqfan)

> **Deprecated.** The conversational backlog + slot-management
> protocol described below is replaced by sqfan's declarative
> `batch.yaml`. Define your tasks once; sqfan handles concurrency
> limits, dispatch ordering, completion metrics, and the polling
> loop. Section kept below for reference and for understanding the
> origin of sqfan's design.

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
  - Record completion metrics:
      ~/repo/dotfiles/scripts/squire-metrics.sh record <env-id>
  - Dispatch next QUEUED item from backlog
  - Report to user
If an env is stalled: send finish prompt.
If backlog is empty and no envs running: kill the loop.
```

No side-table needed in the dispatching session — `squire env info <env-id>` exposes `Created:` which the script parses automatically.

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

## Completion Metrics

Track time-to-completion and diff size for every dispatch so the cost/yield of squire work is visible across the fleet. Without this, the wall-clock estimates in the task-family table stay anecdotal; with it, they get refined by real data.

### What to record

One JSONL line per completed dispatch, captured at extraction time (after the env has committed + pushed, or after bundle extraction). Required fields:

- `env_id` — the squire env ID (`jade-sloth-58345`)
- `env_name` — human-readable name from `squire env`
- `started_at` — ISO-8601 UTC timestamp of when `squire new` ran. Pulled automatically from `squire env info <env-id>` (the `Created:` field) when not supplied; override only if you want to scope duration to a sub-period (e.g. a re-dispatch into an existing env).
- `completed_at` — ISO-8601 UTC timestamp at record time
- `duration_seconds` — `completed_at - started_at`
- `branch` — the env's working branch
- `base` — base SHA the diff is computed against (defaults to `merge-base origin/main <branch>`). For dispatches that EXTEND an existing branch with prior commits, pass `--base <head-before-dispatch>` explicitly, or the recorded LOC/file counts will include the prior work.
- `commit_count` — `git rev-list --count base..HEAD`
- `files_changed`, `lines_added`, `lines_removed` — from `git diff --shortstat base..HEAD`

### Where it lives

Canonical location: `~/repo/dotfiles/scripts/squire-metrics.jsonl`. Override via `SQUIRE_METRICS_FILE` env var if the dotfiles repo isn't checked out at that path.

### The helper script

`~/repo/dotfiles/scripts/squire-metrics.sh` provides two subcommands:

```bash
# Capture a record at completion time. started_at is pulled from
# `squire env info` automatically; override only if needed.
squire-metrics.sh record <env-id>

# Aggregate stats across all recorded dispatches.
squire-metrics.sh tally
squire-metrics.sh tally --last 10
```

The `record` subcommand SSHes into the env to compute diff stats. The `tally` subcommand reports min/median/mean/max duration plus LOC totals, optionally restricted to the last N records.

### When to record

Record at one of these points:

1. **Branch-push completion** — env committed and pushed to a branch the dispatching session controls. Run `squire-metrics.sh record` after the push lands.
2. **Bundle extraction** — env can't push (no GitHub App access); commits extracted via `git bundle`. Run record after the bundle is fetched and reviewed locally.

Both happen at known points in the polling loop. Add a record step to the loop's "env committed and pushed" branch (see Background Agent Polling).

### When NOT to record

- Stalled envs that never committed — they have no diff to measure.
- Envs cut off in drain mode — the work was abandoned; recording duration is misleading.
- Failed envs with no commits — same.

### Tally examples

After enough records accumulate, `tally` answers questions the task-family table currently guesses at:

- Is "small / comment-only" actually 15 minutes? `tally --last 20` over comment-only briefs.
- Does adding the `question`-tool ban shrink duration? Compare tally before/after the rule was added.
- LOC distribution per task family — is the per-family scope creeping?

Refine the task-family table's wall-clock estimates once N >= 5 records exist per family.

## Legacy: Parallel Envs (superseded by sqfan)

> **Deprecated.** Multiple-env work belongs in a sqfan `batch.yaml`,
> not in a sequence of bare `squire new` calls. Section kept for
> single-env one-off context and for understanding what sqfan
> automates.

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

The gateway MCP `create_env` tool is accessible from within an env and exposes creation options (`model`, `flavor`, `git_branch`) for env-to-env delegation when one agent needs to spin up sub-environments. (These are now also on the `squire new` CLI as `--model`/`--flavor`/`--git-branch`; use the MCP form only from within an env.)

## Common Mistakes

- **Using `git clone` URL for repos not in the GitHub App** — the env's credential helper only covers repos the squire GitHub App is installed on. For other repos, use `git bundle` + `scp` (see above). The symptom is `could not read Username for 'https://github.com/...'`.
- **Assuming `gh` CLI is authenticated in the env** — `/usr/bin/gh` IS installed in squire envs (verify with `gh --version`; on 2026-06 images it's v2.93.0), but it is NOT authenticated to any GitHub host. `gh pr diff`, `gh pr view`, `gh api`, etc. all fail with `You are not logged into any GitHub hosts`. There's no token in the env's environment vars either. The symptom is silent if the agent doesn't check stderr, or "not logged in" if it does. **Two correct patterns:**
  1. **Pre-stage everything via scp from the laptop before dispatch.** If the agent needs the PR diff, run `gh pr diff <num> --repo <owner>/<repo> > /tmp/pr-<num>.diff` locally and `scp` it into the env to `/tmp/`. Same for `gh pr view --json title,body`, comments, etc. The dispatched agent reads `/tmp/...` files instead of calling `gh`. This is the cheapest pattern for one-shot tasks like code review where the data is bounded.
  2. **Mint a token via envmgr MCP at dispatch time, export `GH_TOKEN`.** The env's envmgr at `localhost:9877` exposes a `git_token` MCP tool (see `c1-dev-stack-in-squire` for the JSON-RPC protocol). It returns a short-lived GitHub App installation token scoped to the repos the squire GitHub App covers (today: just `ductone/c1`). For tasks that need ad-hoc gh calls, the dispatch prompt should include the token-minting recipe AND export `GH_TOKEN=<minted>` before any `gh` call. Note: the token expires (~30 min); long-running tasks may need to re-mint.

  Choose pattern 1 for code review / diff inspection (bounded, easy to stage). Choose pattern 2 only when the agent needs to make many GitHub API calls or call repos beyond the scope of pre-staging. Never assume `gh auth status` returns logged-in.
- **Wrong model ID in prompt_async** — using a model string like `claude-sonnet-4-20250514` silently fails with `ProviderModelNotFoundError`. The session shows 0 messages and the agent never starts. The correct default is `claude-opus-4-6`. Check the env's config: `cat /home/squire/.config/opencode/opencode.json | jq '.model'`.
- **Wrong prompt payload format** — `{"content": "..."}` does NOT work. The payload requires `messageID`, `role: "user"`, `parts`, and `model`. Missing `role: "user"` is the silent killer: server returns 204, agent errors with "No user message found in stream", model is never invoked. See OpenCode API Protocol above.
- **Talking to port 4096 instead of the env's random port** — the env's authenticated `opencode serve` listens on a randomly-assigned port (e.g. 36127, 43825). Port 4096 only listens if you started your own server. The env's server has `ANTHROPIC_API_KEY` in its process env; yours does not, and your prompts will fail with `ProviderAuthError`. Find the real port with `ss -tlnp | grep opencode` or `pgrep -af "opencode-linux-arm64.*serve"`. See OpenCode API Protocol above for the API-key recovery pattern.
- **Editing prompt JSON in-env with `jq`** — produces output that passes JSON shape checks but is rejected at model invocation. Build the payload locally with the Write tool and `scp` it in.
- **Agent uses the `question` tool and stalls forever** — opencode has no programmatic answer API. Always include "HARD RULE: do not use the `question` tool" in your dispatch prompt. If a session is already stalled, create a fresh session and re-dispatch with the rule embedded.
- **Stopped env not resumable from old `squire` CLI** — squire envs auto-stop on idle and need `squire env start <name>` to wake. Older `squire` CLI builds lack the `start` subcommand (`squire env start` errors with "accepts at most 1 arg(s), received 2"). Upgrade: `make -C ~/repo/squire install`. Disk state is preserved across stop, including commits on the agent's branch. After restart, the env's opencode is back on a possibly-different random port; any port-4096 server you launched needs to be relaunched (with the API key).
- **Polling loop wastes cycles probing dead envs** — add `if env status=stopped, report and stop polling that env` to the polling prompt. Otherwise the cron keeps SSHing into stopped envs and only sees error messages.
- **Agent introduces or references types that live in a sibling repo and ships only the CLI/UI side** — for Latchkey work this happens because the CLI lives in `latchkey-client-shells`, the SDK in `latchkey-client-sdk`, the proto in `latchkey-proto`, the MLS adapter in `latchkey-mls-core`, and the desktop in `latchkey-desktop`. The `[patch]` table the agent committed into `Cargo.toml` (pointing at sibling working trees in `/data/squire/src/`) papered over the gap inside the env, but the missing exports / proto field changes never landed upstream. On a clean clone the consumer fails to compile. Brief the in-env agent explicitly: when a change requires modifying a sibling repo (a new SDK type, a proto field rename, a new MLS adapter method), either (a) make the sibling change in the same dispatch and produce commits against both repos' branches, or (b) stop at the boundary and document the required sibling change in the status note instead of papering over it with a local `[patch]`. The agent must not commit a `[patch]` table that resolves to sibling working trees — that always represents a missing upstream change. For Latchkey specifically, name the canonical repo roots in the prompt so the agent can navigate them: `/data/squire/src/latchkey-client-shells`, `/data/squire/src/latchkey-client-sdk`, `/data/squire/src/latchkey-proto`, `/data/squire/src/latchkey-mls-core`, `/data/squire/src/latchkey-desktop`, `/data/squire/src/c1`.
- **Piping binary data through `squire ssh`** — SSH via squire mangles binary. The pipe `cat file | squire ssh <id> -- "cat > dest"` corrupts non-UTF-8 content. Use `scp <file> <env>.squire:/path` instead.
- **Forgetting `--no-attach`** — without it, `squire new` attaches to the agent TUI. Use `--no-attach` for scripted/parallel workflows. (Older CLIs called this flag `--no-open`; current builds reject `--no-open` with "unknown flag".)
- **Missing quotes around SSH commands** — `squire ssh <id> -- cd /foo && bar` runs `bar` locally. Always quote: `squire ssh <id> -- "cd /foo && bar"`.
- **Stale bd lock from crashed subagents** — if a subagent held the beads DB lock and crashed, `bd` fails with "another process holds the exclusive lock". Fix: `rm /path/to/.beads/embeddeddolt/.lock` (verify no process holds it with `lsof` first).
- **Workspace directory** — OpenCode's working directory defaults to `/data/squire/src`. Older images pre-cloned c1 there; newer images may launch with an empty `src/`. Verify with `ls /data/squire/src/` before dispatching; clone c1 (or any other repo) into its canonical path before sending the prompt. Your prompt must name the full path to the repo (e.g. `/data/squire/src/c1` or `/data/squire/src/occult`). The `directory` field in the session object shows where the agent is actually working.
- **OpenCode log files may not exist** — older images wrote logs to `/home/squire/.local/share/opencode/log/*.log`. Newer images launch `opencode serve` with `--print-logs` only (no file rotation), so that directory is missing and `grep modelID ...log/*.log` returns "No such file or directory". For model-drift verification on those envs, hit the session API instead: `curl -sf http://localhost:<port>/session/<sid>/message | jq '.[-1].metadata.assistant.modelID'` shows the model used on the most recent assistant turn. Always discover the port (`ss -tlnp | grep opencode`) before forming the URL.
- **Model drift mid-session** — OpenCode's whitelist includes cheaper models (haiku, sonnet, GPT variants). The agent can switch models mid-session without warning, producing lower-quality output. Always include the `model` field in every `prompt_async` call and verify the model on polling ticks by grepping `modelID` in the logs. See "Model Enforcement" section above.
