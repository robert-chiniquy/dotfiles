---
name: subagent-prompt-review
description: |
  Review a prompt about to be sent to a subagent (Agent tool, sqfan dispatch,
  squire env, scheduled remote agent, MCP-driven worker) for defects that
  cause silent failure, wasted runs, or runaway scope. Single-axis review:
  is this prompt fit-to-dispatch? Use proactively before any Agent() call
  whose prompt is more than a sentence or two, before `sqfan dispatch`,
  before `/schedule`-style remote-agent creation, and on demand. Triggers
  on: review this prompt, lint this prompt, check this subagent prompt,
  is this prompt OK, prompt review, prompt lint, subagent prompt review,
  before dispatch, before scheduling, vet this delegation, prompt sanity
  check.
---

# Subagent Prompt Review

Single axis: is this prompt fit to dispatch? Subagents start cold — no
parent context, no shared state unless given, no follow-up questions.
Applies to any dispatch surface: Agent tool, `sqfan dispatch` /
`sqfan nudge`, `squire task create`, scheduled remote agents.

Skip trivial one-liners ("run `make test`") — they don't need cwd,
branch, or success criteria, and the review costs more than it saves.

## BLOCKING — run fails or does harm

- Nonexistent model name. Verify against the canonical source, never
  training data:
  - sqfan: `defaultAllowedModels` in
    `/Users/rch/repo/sqfan/pkg/opencode/allowlist.go` plus
    `allowed_models` in `~/.sqfan/config.yaml`.
  - Agent tool `model:` must be `sonnet` | `opus` | `haiku` | `fable`;
    anything else fails with `InputValidationError`.
  - `/schedule`-style remote agents: `claude-{opus,sonnet,haiku}-{version}`
    (e.g. `claude-sonnet-4-6`), checked against a current source.

  Common shapes: recently retired version, digit transposition.
- Embedded credentials: `gh[pousr]_[A-Za-z0-9_]{36,}`,
  `xox[bpor]-[A-Za-z0-9-]+`, `AKIA[0-9A-Z]{16}`, `sk-[A-Za-z0-9]{32,}`,
  `-----BEGIN .* PRIVATE KEY-----`. BLOCKING even for "test" credentials.
- Conflicting or missing destination: "edit `pkg/foo/x.go` in the squire
  env" with no env named; "open a PR" with no repo. The subagent picks
  one at random.
- Destructive action without scope: "delete stale branches",
  "force-push if needed", "rm -rf /tmp/sqfan-*". Narrow the scope or
  strip the authorization.
- Empty or whitespace-only prompt.
- Wrong model class: Haiku for diagnosis/bugfix (fabricates on failure
  paths); Opus for "run `make test`, report green/red".
- `git push` delegated to Haiku — Haiku fabricates code changes to
  satisfy pre-push hooks rather than report failure. Fix: Haiku does
  add+commit, parent pushes.
- Tests expected to fail delegated to Haiku — it misinterprets failures
  and fabricates explanations. Haiku is green-path verification only.

## SHOULD_FIX — do not ship as-is

- Leftover placeholders: `{{X}}`, `<TODO>`, `XXX`, `$VAR` that look
  unsubstituted. `${VAR}` inside a bash code block is intentional
  syntax, not a placeholder.
- Local-machine paths (`/Users/<name>/...`): the subagent's filesystem
  is not the operator's — squire envs, remote schedulers, and MCP
  workers see different layouts.
- Conflicting instructions: "do not modify code" + "fix the bug";
  multiple "you are X" framings; contradictory success criteria.
- Missing repo / branch / cwd for prompts that operate on specific
  files or state.
- Missing success criterion — without a verification gate the subagent
  reports "done" when it stops, not when the work is correct.
- Haiku on a read-only task without a "do NOT modify files" guard —
  Haiku "fixes" code unprompted.
- Missing "do NOT commit/push" guard when that authority was not
  delegated.
- Stale references (bd-id, PR number, path). The subagent cannot repair
  a broken reference; it fabricates or fails opaquely. Verify before
  dispatch.
- Tool/command references that don't exist (`sqfan foo`, `bd bar`) —
  verify against the actual CLI surface.

## NIT

- Banned phrases / business-speak per global rules: "monadic",
  "bikeshedding", "key insight", "leverage", "synergy", "ROI", "KPI",
  "action item", "circle back", "bandwidth".
- Mirror-style scaffolding ("I need you to think about...") — state
  the task directly.
- Excess background the subagent must read past.
- No output shape when the parent consumes the result. E.g. "Report
  PASS/FAIL with last 30 lines on failure".

## Out of scope

Whether the task itself is well-conceived; code style inside code
blocks embedded in the prompt; the subagent's eventual output.

## Output

```
BLOCKING (N)
  - <span>: <defect>. Fix: <concrete change>.
SHOULD_FIX (N)
  - ...
NIT (N)
  - ...
```

Quote the exact offending span so the operator can grep for it. Fixes
must be concrete — "rephrase for clarity" is not a fix. End with one
verdict line:

- `READY` — no BLOCKING, no SHOULD_FIX.
- `READY WITH NITS` — only NITs.
- `NEEDS REVISION` — any SHOULD_FIX, no BLOCKING.
- `DO NOT DISPATCH` — any BLOCKING.
