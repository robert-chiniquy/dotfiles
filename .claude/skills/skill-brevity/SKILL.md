---
name: skill-brevity
description: Length discipline for skill authoring. Use whenever writing a new skill, editing an existing SKILL.md, or reviewing/minimizing a skill collection.
---

# Skill brevity

For each line ask: would a competent model do this unprompted? If yes, cut it.

- Keep: domain facts, repo-specific paths and names, non-obvious constraints,
  trigger phrases, hard-won gotchas (the "Common Mistakes" class).
- Cut: process narration, generic best practices, restated model defaults,
  hedges, and instructions that only explain why other instructions exist.
- Prefer deleting a questionable line over qualifying it.
- Before trusting a cut, verify against the skill's core scenario — run it or
  walk it — rather than judging by whether the line "looks important."
- Skills written for older models are usually too prescriptive; trim on touch.
- Preserve the original as SKILL.md.orig in the same directory when doing a
  deliberate minimization pass (only SKILL.md is loaded, so .orig is inert).
