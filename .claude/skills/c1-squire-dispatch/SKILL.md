---
name: c1-squire-dispatch
description: >-
  c1-specific values for the general squire dispatch protocols defined in
  squire-env-management. Provides the c1 gate bundle's contents, the
  task-family table for c1 work, the c1 always-actives, and the list of
  c1 skills that should NOT be spent on a squire env. Use when about to
  spawn a squire env to execute c1 work, when writing a brief for a
  remote c1 agent, or when filing a c1 bead intended for squire dispatch.
  Triggers: c1 squire dispatch, c1 squire brief, c1 remote work,
  c1 ephemeral env, c1 fire-and-forget.
---

# c1 squire dispatch

The c1-specific instantiation of the protocols defined in `squire-env-management`. That skill provides the *shape* — gate bundle pattern, brief-templates concept, beads manifest format, failure debrief protocol. This skill provides the c1 *content* that fills those shapes.

Read `squire-env-management` for the protocol mechanics. Read this skill for what to plug into them.

Pairs with `c1-dev-stack-in-squire` (the env shape for tasks that need a running c1 backend).

## c1 Always-Actives

Every c1 squire brief implicitly loads these on top of the task-family row:

- `security-patterns` — MUST when touching auth, authz, context propagation, sensitive data
- `pgdb-index-coverage` — MUST when adding WHERE clauses or proto indexes
- `go-conventions` — when writing Go in `pkg/` or `cmd/`
- `code-search` — for navigation

## c1 Gate Bundle

Resolves `Gates: standard` for c1. The remote agent expands to the relevant subset based on file types changed in the diff.

| Gate | Skill | When |
|---|---|---|
| Lint Go | `lint-go` | Go files changed in `pkg/` or `cmd/` |
| Lint frontend | `lint-frontend` | TS/TSX files changed |
| Test changed packages | `test-changes` | Always |
| Validate protos | `validate-protos` | `.proto` files changed |
| Post-change verify | `post-change-verification` | Always for Go |

## c1 Task Family Table

Resolves `Skills: standard` and `Env:` for c1. Pick one row per dispatch.

### Backend / Go

| Task | Skills | Env |
|---|---|---|
| New CRUD endpoint | `new-crud`, `policy-filtering`, `pgdb-index-coverage` | minimal |
| New temporal workflow | `new-temporal-workflow`, `temporal-workflows` | c1-dev-stack |
| New migration | `new-migration`, `uplift-patterns`, `pgdb-index-coverage`, `validate-protos` | c1-dev-stack |
| New feature flag | `new-feature-flag` | minimal |
| New quota | `new-quota` | minimal |
| MCP API object | `mcp-api-objects` | minimal |
| Port legacy CTE | `port-cte`, `pgdb-index-coverage` | minimal |
| Retire feature flag | `retire-feature-flag` | minimal |
| Sync subsystem work | `sync-architecture` + relevant row above | c1-dev-stack |

### Frontend

| Task | Skills | Env |
|---|---|---|
| New / changed component | `component-review`, `document-component` | minimal |
| Migrate route | `migrate-routes`, `component-review` | minimal |
| UI copy review | only the `ui-copy-*` skills matching widgets actually touched | minimal |
| Component verification in browser | `agent-browser` (default) or `playwright-browser` (auth-tricky), `dev-util` | c1-dev-stack |

Do NOT load all 16 `ui-copy-*` skills. Pick by widget type touched (button, modal, toast, empty-state, etc.).

### Verification (no code changes, just driving a running stack)

| Task | Skills | Env |
|---|---|---|
| End-to-end UI flow | `agent-browser`, `dev-util` | c1-dev-stack |
| Backend integration check | `dev-util` | c1-dev-stack |

### Connector work

| Task | Skills | Env |
|---|---|---|
| New / modified Baton connector | `connector` | minimal (or c1-dev-stack if testing against pub-api) |

## Main-Session-Only c1 Skills (do NOT dispatch to squire)

Run these in the dispatching session against the returned PR or against PR history. They do not need a squire env:

- `ci-review` — when the squire-produced PR comes back red.
- `add-release-notes` — drafted on the returned PR before merge.
- `report-release-notes` — weekly recurring; reads merged PR list, not local repo state. A scheduled agent, not a squire dispatch.
- `component-review` against returned diff — fast diff-only review.
- Relevant `ui-copy-*` skills — applied to the returned frontend diff.

## What this skill is NOT

- Not the dispatch protocol — see `squire-env-management`.
- Not policy for the c1 repo — this is personal workflow. Do not commit it into c1's `.claude/`.
- Not a substitute for reading the individual c1 skill SKILL.md files — the tables list names; the skills themselves carry the details.
