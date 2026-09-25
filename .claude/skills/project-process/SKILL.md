---
name: project-process
description: >
  Design → implementation-plan → RFC pipeline and optional local artifacts.
  Load when starting non-trivial work, planning, writing DESIGN/PLAN, or
  choosing whether an RFC is needed. Not always-on.
---

# Project process

Prefer working code and verified plans over ceremony.

## Common Mistakes

1. **Skipping stages under "just implement"** — if `DESIGN_*.md` exists, the plan MUST consume it; if `PLAN_*.md` exists, RFC investigation MUST consume it.
2. **Inventing a parallel design method** — one path only (table below). Do not blend pqthink + systematic-feature-design + freeform essays as competing pipelines.
3. **Creating GLOSSARY.md or DATA_SOURCES.md** — forbidden; terms in prose; provenance in README `## Sources`.
4. **Committing local meta in shared repos** — LEARNINGS/PLAN/FAILURES stay local-only unless the user says otherwise. On first touch in a repo, check `git ls-files` and `git check-ignore` for them: a tracked one is untracked (`git rm --cached`, file stays on disk, back it up first since the next pull of a deleting commit removes an unmodified copy) and every one is listed in `.gitignore` in the same change.
5. **Auto-starting implementation from an RFC skill** — RFC ends at owner approval.
6. **Squire brief as a shell `-p` or a stray path** — long briefs go in `plans/<topic>/` next to the RFC/plan, then `squire task create --prompt-file`.
7. **Chat path without opening the artifact.** Presenting a DESIGN, PLAN, RFC, guide, report, proposal, or other single handed-over document without `code <absolute-path>` (VS Code) or `open <absolute-path>` (OS default, e.g. mp4) in the same turn. Path in chat is not enough.

## Artifacts (when useful)

| Place | When |
|-------|------|
| `LEARNINGS.md` | Discovery that would otherwise be lost |
| README `## Sources` | External / cross-repo provenance |
| `FAILURES.md` | Approach proved impossible |
| `DESIGN_<topic>.md` | Stage 1 |
| `PLAN_<OBJECTIVE>.md` | Stage 2 |
| `plans/<topic>/` | Plan, RFC, and squire `--prompt-file` brief together |
| `.claude/CLAUDE.md` | Project-local rules only |

## Pipeline (one path)

```
DESIGN_<topic>.md  →  PLAN_<OBJECTIVE>.md  →  RFC (new-rfc)
     stage 1                 stage 2              stage 3
```

| Stage | Produce | Load exactly one depth skill | Stop when |
|-------|---------|------------------------------|-----------|
| 1 Design | `DESIGN_*.md` | default: `design`; large/architecture: `systematic-feature-design` only | Implementer can plan without re-discovering the problem |
| 2 Plan | `PLAN_*.md` | `rigorous-critique` (manual `/critique` is the same job) | Steps executable without re-arguing design |
| 3 RFC | approved RFC | `new-rfc` only | Owner approved; **MUST NOT** auto-impl. May include a retcon element (`$retcon`, `$retcon-review`) when the artifact will be shared. |

**MUST:** each stage consumes the prior artifact when it exists.  
**MUST NOT:** open stage 3 without feeding design+plan (or a combined doc that includes both).

| Situation | Stages |
|-----------|--------|
| Typo / mechanical fix | none |
| Clear feature, one owner | 2; add 1 if ambiguous |
| Large / multi-system / high risk | 1 → 2 → 3 |
| User asks for RFC only | 3, but inject any existing DESIGN/PLAN |

## Hard gates

- **MUST** write plans to `PLAN_<OBJECTIVE>.md` (not only chat). Large work: `plans/<topic>/` holds the plan, the RFC, and the squire prompt file together.
- **MUST** run critique before treating a plan as ready for implementation or RFC.
- **MUST NOT** treat "out of scope" as forbidden forever without a revisit path (see overcorrection-review when cutting).
- **MUST** put independently dispatchable leaves in the RFC as a work-unit table (`id`, `isolation`, `verifier`, `tier`, `collides-with`) and `plans/<topic>/units.yaml` measured by `squine lint`. Parallel means empty `collides-with`. Tests are the `verifier` column; CHEAPEST/MEDIUM assignment is route at dispatch.

## Before finishing (pipeline work)

- [ ] Correct stage for the situation?
- [ ] Prior-stage artifact read and reflected?
- [ ] No forbidden meta files (GLOSSARY/DATA_SOURCES)?
- [ ] Local meta not staged for commit in a shared repo?
- [ ] Next stage (or implementation) is explicit?
- [ ] Presented artifact opened with `code <absolute-path>` (or `open` if VS Code is the wrong viewer)?

Optional depth: `references/` (artifacts, priorities, organization).  
Judgment on demand: `engineering-guidelines`.
