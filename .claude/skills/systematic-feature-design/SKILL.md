---
name: systematic-feature-design
description: |
  Pipeline stage 1 depth only — large feature / architecture design.
  Not a parallel pipeline to /design. Use when the problem is large; otherwise
  use design. Writes DESIGN_<topic>.md for stage 2. See project-process.
---

# Systematic Feature Design

**Pipeline stage 1 (large only).** See `project-process`. **MUST** write
`DESIGN_<topic>.md`. **MUST NOT** also run full `/design` as a second process.

## Common Mistakes

1. Using this for small features (use `design` instead).
2. Skipping written DESIGN output (chat-only design dies at stage 2).
3. Treating critique/double-back inside this skill as a substitute for stage 2
   `rigorous-critique` on the plan.

## Before finishing

- [ ] `DESIGN_*.md` exists and is consumable by PLAN?
- [ ] Level 0 before Level 2?
- [ ] Existing RFCs/plans searched before inventing?

11-step methodology (research → … → document). Critique steps here inform the
design; stage 2 still runs `rigorous-critique` on the plan.

Steps: Research, Compare, Refine (qualitative to quantitative), Design Documentation Ontology, Ideate, Discover (existing codebase), Implement (detailed paths), Critique, Double Back, Consolidate, Document Decisions.

Key principles:
- Steps 7-8 (Critique and Double Back) create the most value
- Level 0 (platform) before Level 2 (polish)
- Concrete over abstract -- every feature needs examples
- Additive documentation -- never delete research
- Vertical slices -- ship value incrementally
- Existing RFCs are first-class design evidence: when an open design question appears, search repo-local RFCs/ADRs/proposals/decision notes for a pre-existing path and raise matching artifacts to the user as candidate directions before inventing a new answer.

Uses the Phase Framework for document versioning (PHASE_1_RESEARCH.md, etc.) and the Level Framework (Level 0: Platform, Level 1: Workflow, Level 2: Polish).
