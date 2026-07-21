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

The app under verification owns `.verify/workflows.yaml` (manifest of named
flows), a runner command template, and whatever executes flows (Playwright,
HTTP, curl). The manager only reads the manifest, invokes the runner, and
parses its stdout JSON. Not a substitute for unit/integration tests.

## Manifest

`.verify/workflows.yaml`:

```yaml
version: 1
# {flow} is replaced with the flow name. Must print one JSON object to stdout,
# exit 0 on pass / non-zero on fail.
runner_command: "npm run verify -- {flow}"
base_url: "http://localhost:3000"  # optional; runner may also read VERIFY_BASE_URL
flows:
  - name: checkout-happy-path
    description: Anonymous user adds an item to cart and reaches checkout.
    timeout_s: 60
    tags: [critical, checkout]
```

## Discovery

Substitute `--list` for `{flow}` in `runner_command`. Stdout:
`{"flows": [{name, description, timeout_s, tags, implemented}]}`.
Prefer `--list` over parsing the manifest YAML — the runner is the interface.
`implemented: false` = declared but no backing code; surface as a setup error,
do not run.

## Runner output contract

One JSON object to stdout; logs, progress, and browser chatter to stderr.

```json
{
  "flow": "checkout-happy-path",
  "status": "pass",
  "duration_ms": 4320,
  "steps": [
    { "name": "add-to-cart", "status": "pass", "duration_ms": 1200 },
    { "name": "assert-cart-count-1", "status": "fail", "duration_ms": 50,
      "error": "expected 1, got 0" }
  ],
  "artifacts": {
    "screenshot": "/tmp/verify/checkout-happy-path.png",
    "url_at_failure": "http://localhost:3000/cart"
  }
}
```

- Exit 0 = pass, non-zero = fail. Exit code and `status` must agree;
  disagreement = failure.
- A runner that crashes before printing can leave stdout blank with non-zero
  exit — check both; either signals failure.
- Unparseable stdout = failure; report raw stderr.

## Procedure

Given repo `$DIR` and flow `$FLOW`:

1. Read `$DIR/.verify/workflows.yaml` for `runner_command`. No manifest →
   skill does not apply; abort.
2. Run `--list` with `cwd=$DIR`; parse stdout JSON.
3. Confirm `$FLOW` is listed with `implemented: true`.
4. Substitute `{flow}` with `$FLOW`; execute with `cwd=$DIR`. Capture stdout,
   stderr, exit code. Enforce the flow's `timeout_s`.
5. Parse stdout JSON; report pass/fail plus the first failing step's error.

## Issue-tracker integration

An issue description may declare flows to verify in a `Verify:` section
(one flow name per bullet). When a subagent reports the task complete: read
the issue (`bd show <id>` or equivalent), parse `Verify:`, run the procedure
per flow. Any failure → post the failure JSON as a comment, leave the issue
open. All pass → close.

## Page objects (reference implementation)

Runner may be any language; `reference/` is TypeScript + Playwright with page
objects:

```
.verify/
├── workflows.yaml
├── package.json
└── src/
    ├── runner.ts    # dispatch by flow name, emit JSON
    ├── pages/       # one class per page/screen; selectors live here
    └── flows/       # one module per flow; drives page objects
```

Selectors live in page objects, intent in flows; the manager never sees
selectors or DOM, only named steps and results.

## Common Mistakes

- Writing Playwright or driving a DOM from the manager. The manager never
  spawns a browser — the runner is the seam.
- Adding assertions in the manager. Assertions live in flows; the manager's
  only judgment is pass/fail + which step failed.
- Synthesizing flows on the fly. Flows are additions to the app repo; if a
  flow isn't in the manifest, ask for it to be added — do not improvise one.
- Inferring success from logs or partial output. Exit code and JSON `status`
  are the only sources of truth; malformed output is itself the failure.
