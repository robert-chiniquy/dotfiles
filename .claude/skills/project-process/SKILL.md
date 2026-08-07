---
name: project-process
description: >
  Thin project process: optional local artifacts, and the design →
  implementation-plan → RFC pipeline. Always active as a hub; load
  references and sibling skills only when the stage requires them.
---

# Project process

Keep process light. Prefer working code and verified plans over ceremony.

## Artifacts (create when useful, not by default)

| File | When |
|------|------|
| `LEARNINGS.md` | You discovered something that would otherwise be lost |
| `DATA_SOURCES.md` | External or cross-repo inputs inform the work |
| `GLOSSARY.md` | Domain terms of art need a shared definition |
| `FAILURES.md` | An approach proved impossible |
| `DESIGN_<topic>.md` | Stage 1 output (below) |
| `PLAN_<OBJECTIVE>.md` | Stage 2 output (below) |
| `.claude/CLAUDE.md` | Project-local rules (never global prefs that belong in home Claude.md) |

In shared repos (has a git remote), meta docs above are **local-only** — do not commit unless the user says so. See `references/artifacts.md` only if you need templates or edge cases.

## Pipeline: design → plan → RFC

Use this order for non-trivial work. **Each stage consumes the prior stage's artifact.** Do not invent a later stage that ignores an earlier one that exists.

```
DESIGN_<topic>.md  →  PLAN_<OBJECTIVE>.md  →  RFC (new-rfc skill)
     stage 1                 stage 2              stage 3
```

### Stage 1 — Design

**Goal:** problem, constraints, options, chosen direction, open questions.  
**Produce:** `DESIGN_<topic>.md` (or equivalent path the user names).  
**How:** `/design` skill, or `systematic-feature-design` when the problem is large.  
**Stop when:** a competent implementer could draft a plan without re-discovering the problem.

Skip for trivial bugfixes and mechanical renames.

### Stage 2 — Implementation plan

**Goal:** dependency-ordered steps, vertical slices, success criteria, explicit non-goals.  
**Consumes:** the design doc (required if one exists).  
**Produce:** `PLAN_<OBJECTIVE>.md`.  
**How:** plan from the design; run `critique` / `rigorous-critique` before treating the plan as ready.  
**Stop when:** steps are executable without re-arguing the design.

If design was skipped, the plan must still state problem + constraints in a short header so stage 3 (if used) has input.

### Stage 3 — RFC

**Goal:** adversarially reviewed, owner-approved plan for work that needs rigor, multi-party buy-in, or a durable decision record.  
**Consumes:** design + implementation plan (or a single combined doc that clearly includes both).  
**Produce:** approved RFC via the `new-rfc` skill.  
**Stop when:** owner approves; **do not** auto-start implementation from this skill.

Skip RFC for small solo work where plan + critique is enough. Prefer RFC when the change crosses trust boundaries, migrations, public API, or multi-repo coordination.

### Stage selection

| Situation | Stages |
|-----------|--------|
| Typo, lint, tiny fix | none |
| Clear feature, one owner | plan (2); design if ambiguous |
| Large / multi-system / high risk | 1 → 2 → 3 |
| User asks for RFC only | run `new-rfc`, but feed any existing DESIGN/PLAN in as investigation inputs |

## Priorities (short)

1. Unblock verified delivery over perfect docs.  
2. Prefer parallel independent work when dependencies allow.  
3. Record learnings as you go; do not batch them at the end.  

Detail on sequencing: `references/priorities.md` (optional).  
Organization / multi-package layout: `references/organization.md`, `references/multisubproject.md` (optional).

## Related skills (load by stage)

| Stage | Skills |
|-------|--------|
| Design | `design`, `systematic-feature-design`, `socratic-discovery` |
| Plan | `rigorous-critique`, `critique`, `complete-developer-experience` |
| RFC | `new-rfc` |
| Judgment | `engineering-guidelines` (on demand) |
