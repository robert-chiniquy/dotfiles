---
name: skill-brevity
description: Length discipline for skill authoring. Use whenever writing a new skill, editing an existing SKILL.md, or reviewing/minimizing a skill collection.
---

# Skill brevity

For each line ask: would a competent model do this unprompted? If yes, cut it.

## Compliance shape (MUST for long skills)

1. After title/1-paragraph intro: **Common Mistakes** (or Gates) first.
2. End high-stakes skills with **Before finishing** (≤5 checkboxes).
3. Gates use MUST / MUST NOT — not prefer/should when the rule is absolute.
4. One procedure per job; link to `project-process` rather than inventing parallel methods.

## Cut / keep

- Keep: domain facts, repo-specific paths and names, non-obvious constraints,
  trigger phrases, hard-won gotchas (Common Mistakes).
- Cut: process narration, generic best practices, restated model defaults,
  hedges, and instructions that only explain why other instructions exist.
- Prefer deleting a questionable line over qualifying it.
- Before trusting a cut, verify against the skill's core scenario.
- Skills written for older models are usually too prescriptive; trim on touch.
- Preserve SKILL.md.orig only for deliberate minimization (not loaded).
