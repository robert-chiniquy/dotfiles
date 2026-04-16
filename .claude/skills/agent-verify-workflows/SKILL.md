---
name: agent-verify-workflows
description: >-
  Explicit protocol for a management agent to verify web workflows after a
  subagent completes a task, without writing Playwright or browser automation.
  Trigger only when the user explicitly asks to verify workflows, run a named
  flow, or mentions agent-verify / verify-workflows / page-objects-for-agents.
  Do NOT trigger on general web app work, testing discussions, or code review.
---

# Agent Verify Workflows

A contract for a management agent to verify web workflows owned by an app,
without writing browser automation. The app ships a manifest of named flows
and a single runner command. The manager discovers, invokes, and interprets.

## When to use

Invoke explicitly when:
- A subagent (Squire, Haiku, etc.) reports a task complete and the tracked
  issue lists flows to verify.
- The user asks to verify a specific named flow.
- Confirming a deployment still passes its declared flows.

Do not invoke:
- For general "does the code work" questions.
- As a substitute for unit or integration tests.
- When no manifest exists in the target repo.

## Pattern shape

The app being verified owns three things:

1. `.verify/workflows.yaml` — a manifest listing named flows.
2. A runner command template that takes a flow name as input.
3. Whatever machinery actually executes flows (Playwright, HTTP, curl).

The manager only ever reads (1), executes (2), and parses (3)'s stdout JSON.

## Manifest schema

`.verify/workflows.yaml`:

```yaml
version: 1

# Command template the manager invokes. {flow} is replaced with the flow name.
# Must print a single JSON object to stdout and exit 0 on pass / non-zero on fail.
runner_command: "npm run verify -- {flow}"

# Optional default base URL. Runner may also read VERIFY_BASE_URL from env.
base_url: "http://localhost:3000"

flows:
  - name: checkout-happy-path
    description: Anonymous user adds an item to cart and reaches checkout.
    timeout_s: 60
    tags: [critical, checkout]

  - name: auth-login
    description: Valid credentials route to the dashboard.
    timeout_s: 30
    tags: [critical, auth]
```

## Discovery via `--list`

The runner is the authoritative source for available flows. The manager
substitutes `--list` in place of `{flow}` in `runner_command` and invokes:

```
npm run verify -- --list
```

Stdout:

```json
{
  "flows": [
    {
      "name": "checkout-happy-path",
      "description": "Anonymous user adds an item to cart and reaches checkout.",
      "timeout_s": 60,
      "tags": ["critical", "checkout"],
      "implemented": true
    }
  ]
}
```

`implemented: false` means the flow is declared in the manifest but has no
backing code. Surface this as a setup error — do not attempt to run.

Prefer `--list` over parsing the manifest directly. The manifest YAML is a
declaration; the runner is the interface.

## Runner output contract

The command specified by `runner_command` MUST print one JSON object to stdout.
Logs, progress, and browser chatter go to stderr.

```json
{
  "flow": "checkout-happy-path",
  "status": "pass",
  "duration_ms": 4320,
  "steps": [
    { "name": "navigate-home",       "status": "pass", "duration_ms": 820 },
    { "name": "add-to-cart",         "status": "pass", "duration_ms": 1200 },
    { "name": "assert-cart-count-1", "status": "fail", "duration_ms": 50,
      "error": "expected 1, got 0" }
  ],
  "artifacts": {
    "screenshot": "/tmp/verify/checkout-happy-path.png",
    "url_at_failure": "http://localhost:3000/cart"
  }
}
```

- Exit 0 = pass, non-zero = fail.
- Exit code and `status` must agree. If they disagree, treat as failure.
- If stdout is not parseable as JSON, treat as failure and report raw stderr.

## Manager procedure

Given a target repo directory `$DIR` and a flow name `$FLOW`:

1. Read `$DIR/.verify/workflows.yaml` to obtain `runner_command`. If the
   manifest is missing, abort — this skill does not apply to the target.
2. Discover flows: substitute `--list` for `{flow}` in `runner_command`,
   execute with `cwd=$DIR`, parse stdout JSON.
3. Confirm `$FLOW` exists in the listed flows AND has `implemented: true`.
4. Substitute `{flow}` in `runner_command` with `$FLOW`.
5. Execute with `cwd=$DIR`. Capture stdout, stderr, and exit code. Enforce
   the flow's `timeout_s`.
6. Parse stdout as JSON.
7. Report pass/fail plus the first failing step's error to the user or the
   upstream issue.

Do not layer logic on top of the runner. If the runner's output is malformed,
surface that as the failure — do not attempt to infer success from partial
output.

## Issue-tracker integration

An issue's description may declare which flows to verify after the issue is
resolved. Recommended convention, readable by any tracker:

```
...task description...

Verify:
- checkout-happy-path
- auth-login
```

When the subagent reports task complete:

1. Read the issue (`bd show <id>`, or equivalent).
2. Parse the `Verify:` section for flow names.
3. For each flow, run the manager procedure above.
4. If any flow fails, post the failure JSON as a comment and leave the issue
   open. If all pass, close the issue.

## Page Objects (reference implementation)

Adopters may implement the runner in any language. The reference in
`reference/` uses TypeScript + Playwright with the classic Page Object
pattern to keep flows readable and selectors stable.

```
.verify/
├── workflows.yaml
├── package.json
└── src/
    ├── runner.ts          # dispatch by flow name, emit JSON
    ├── pages/             # one class per page/screen; selectors live here
    │   └── CheckoutPage.ts
    └── flows/             # one module per flow; drives page objects
        └── checkout-happy-path.ts
```

The page object pattern matters because the manager agent never sees
selectors or DOM. It only sees named steps. Selectors live in page objects,
intent lives in flows, and the manager sees results.

## Common Mistakes

- **Writing Playwright in the manager's context.** The manager never spawns
  a browser. It invokes the runner and parses JSON. If you catch yourself
  driving a DOM from the manager, stop — the runner is the seam.

- **Auto-triggering this skill.** The description is narrow by design. If
  the user did not explicitly ask for workflow verification, do not invoke.

- **Inferring success from logs.** The runner's exit code and JSON `status`
  field are the only sources of truth. Treat everything else as noise.

- **Adding assertions in the manager.** Assertions belong inside flows, in
  the runner. The manager's only judgment is pass/fail + which step failed.

- **Synthesizing flows on the fly.** Flows are additions to the app repo,
  not manager-time inventions. If the user wants to verify something not in
  the manifest, ask them to add a flow — do not improvise one.

- **Mixing stdout and stderr.** JSON to stdout, everything human to stderr.
  Mixing breaks the parse.

- **Trusting status without checking exit code.** A runner that crashes
  before printing a result can leave stdout blank with a non-zero exit.
  Check both; either signals failure.
