---
name: rigorous-critique
description: |
  Pipeline stage 2 critique — canonical. Use after PLAN_*.md exists, before
  implementation or RFC. /critique is the same job. Not a design-stage
  substitute. See project-process.
---

# Rigorous Critique

**Pipeline stage 2 (canonical critique).** **MUST** run before treating a plan as
ready. Consumes `PLAN_*.md` (and design if present). `/critique` = this job.

## Common Mistakes

1. Critiquing only in chat with no plan file.
2. Softening findings to keep momentum (blocking stays blocking).
3. Using this as stage 1 design (wrong stage).

## Before finishing

- [ ] Each finding: lens, problem, risk, concrete fix?
- [ ] Plan updated or explicitly rejected?
- [ ] Overcorrection lens applied to proposed cuts?

Four lenses:
1. **Unnecessary Complexity** -- What can we delete without losing value?
2. **Missing Fundamentals** -- Are we building polish before platform? (Level 0 before Level 2)
3. **Feasibility and Value** -- Is the juice worth the squeeze?
4. **Overcorrection** -- Apply `$overcorrection-review`: are cuts, guardrails,
   or exclusions based on questionable estimates or broader than the failure?

Good critique may remove, restore, stage, or narrow a choice. Every finding
names the invariant it preserves and a proportionate alternative.
