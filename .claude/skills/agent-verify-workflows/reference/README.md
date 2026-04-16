# Reference implementation: agent-verify-workflows

Drop `.verify/` into the root of the app being verified. Customize the
manifest, page objects, and flows.

## Structure

- `workflows.yaml` — manifest. Lists named flows and the runner command.
- `src/runner.ts` — dispatches a flow by name, prints JSON to stdout.
- `src/pages/` — page objects. One class per page or screen. Selectors
  live here. Methods expose user intent, not DOM details.
- `src/flows/` — one module per flow. Default export is a function that
  takes a `VerifyContext` and throws on assertion failure.

## Running a flow locally

```
npm install
npm run verify -- checkout-happy-path
```

Output: a single JSON object with per-step results. Exit 0 on pass,
non-zero on fail.

## Listing available flows

```
npm run verify -- --list
```

Output: a JSON object with a `flows` array. Each entry includes name,
description, timeout, tags, and an `implemented` flag. Use this to
discover what can be run; the manifest is the declaration, this is the
interface.

## Adding a flow

1. If the flow touches screens without an existing page object, add one
   under `src/pages/`. Put selectors inside. Expose intent-level methods
   (`addItemToCart`, not `clickButton`).
2. Add a flow under `src/flows/<name>.ts`. Default-export a function that
   drives page objects and wraps each observable step in `ctx.step()`.
3. Add an entry to `workflows.yaml` under `flows:`.
4. Verify locally: `npm run verify -- <name>`.

## What the manager agent sees

The manager reads `workflows.yaml`, picks a flow name, substitutes into
`runner_command`, and executes. It parses the JSON from stdout and reports
pass/fail. It never touches Playwright, DOM, selectors, or browser logs.

That separation is the whole point: the app owns the automation, the
manager owns the orchestration.
