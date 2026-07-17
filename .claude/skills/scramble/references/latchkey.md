# Latchkey scramble defaults

Load when scrambling against the Core Workflow Completion milestone or Latchkey stacks.

## Option-board seeds

When building the scramble plan, these are typical **parallel local options** for Latchkey milestones:

- Rebase + push open c1 latchkey PR; re-request phoebesimon + madison-c-evans
- `make protogen` on any branch with proto conflicts (main clone, not broken worktree)
- Push + open draft PR for branches that exist on 4 repos but have no PR
- Linear state hygiene on mis-bucketed issues (no comments)
- sqfan dispatch for IGA-2446-style adjacent Todo work
- Pin SDK `Cargo.toml` proto rev after latchkey-proto push

Remote-blocked (board only, don't execute): human review on open c1 PR, merge approval.

## Active milestone (as of 2026-07-06)

| Field | Value |
|---|---|
| Name | Core Workflow Completion |
| Linear ID | `596ed52f-64b6-41e8-af2e-835ca7ebc0b6` |
| Target | 2026-07-07 |

## Repo chain (typical order)

```
latchkey-proto → c1 → latchkey-client-sdk → multipass-cli
```

Local paths (discover at runtime if missing):

- `/Users/rch/repo/c1`
- `/Users/rch/repo/latchkey-proto`
- `/Users/rch/repo/latchkey-client-sdk`
- multipass-cli — clone on demand if needed

## Protogen

- **c1:** `make protogen` from repo root (docker-backed; use main clone, not broken worktrees)
- **latchkey-proto:** proto-only; c1 protogen is authoritative for server codegen
- Do **not** block on squire protogen — local pass is the scramble default

## Latchkey PR reviewers (c1)

Per **c1-squire-dispatch** — latchkey-touching PRs:

- `phoebesimon`
- `madison-c-evans`

Do not add default c1 reviewers (`arreyder`, `mj-palanker`) to latchkey PRs.

## Linear API

```bash
source <project>/.env   # LINEAR_API_KEY
# State IDs (IGA team) — re-fetch if stale:
# Todo:     dd9e0cc6-d189-40b4-be3c-6fdaf48fcf87
# In Progress: 9d2703fc-e934-4c77-aea9-0083ee2ade27
# In Review:   6bf4774c-fb37-45a5-b061-faefd083d281
# Done:        34d7b89f-1b8d-4378-be09-8797cab3c70d
```

State updates only during scramble — no comments.

## Sqfan batch layout

```
sqfan-batches/<issue-slug>/
  batch.yaml
  prompts/
    <task>.txt
```

Copy from an existing fleet (e.g. `iga-2416-2417-review-fleet` or `iga-2446-display-context`).

## Todo → fleet suitability (Latchkey-specific)

| Issue cluster | Dispatch? |
|---|---|
| IGA-2446 display context | Yes — pairs with cross-device stack |
| IGA-2558 Linux client | No — platform matrix |
| IGA-1912 / IGA-2245 binding decisions | No — decision first |
| IGA-2630 Secure Enclave | No — hardware/platform |
| IGA-1930 / IGA-2259 / IGA-2260 identity binding | Later — research spikes after display context |

## In Review closure checklist

Issue stays **In Review** until **all** layers merged. Then → **Done**.

Example delete stack (IGA-2416/2417): proto + CLI + SDK merged, c1 open → still In Review.