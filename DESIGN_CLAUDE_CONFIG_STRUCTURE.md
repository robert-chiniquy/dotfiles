# Design: Claude Config Structure

Analysis of the current `.claude/` configuration structure and identification of structural problems that degrade Claude's effectiveness.

## The Core Problem

The current structure conflates three distinct concerns:

1. **Session requirements** - Things Claude must always do (tone, permissions, no emoji)
2. **Domain skills** - Knowledge applicable when relevant (terraform, proto, documentation)
3. **Process methodology** - How to run projects (DATA_SOURCES, checkpoints, etc.)

These are mixed together in ways that cause Claude to:
- Load irrelevant context (wastes tokens)
- Miss relevant context (produces worse output)
- Receive contradictory instructions (from duplicated rules)
- Lose track of what's authoritative

---

## Structural Problems

### 1. CLAUDE.md is a Dumping Ground

The global `~/.claude/CLAUDE.md` is 350+ lines containing:
- Hard requirements (no emoji, no co-authored-by)
- Soft preferences (dark mode, vaporwave colors)
- Process methodology (DATA_SOURCES, versioning, checkpoints)
- Permissions (what commands need approval)
- Loading instructions (which skills to read)

**Problem:** Everything lives at the same priority level. Claude must parse all 350 lines at every conversation start to extract what matters for the current task.

**Symptom:** Rules get missed because they're buried. Rules contradict each other because they were added at different times. New rules keep getting appended because there's no clear place for them.

### 2. Duplication Creates Contradiction Risk

The same content appears in multiple places:

| Topic | In CLAUDE.md | In Skill |
|-------|--------------|----------|
| File versioning | "When receiving feedback..." | `project-practices.md:7-13` |
| Stable identifiers | "Once an item has a number..." | `project-practices.md:17-23` |
| Checkpoint commits | "Create proactively when..." | `project-practices.md:25-31` |
| DATA_SOURCES rules | "Every entry must trace to..." | `project-artifacts.md:26-27` |
| Git commit policy | "meta-documentation has different..." | `project-artifacts.md:179-228` |

**Problem:** When you asked me to update a rule, which location did I update? Probably only one. Now they're out of sync.

**Example of actual drift:**
- CLAUDE.md says `FILENAME_V2.md` (suffix)
- project/SKILL.md:331-336 says `V2_FILENAME.md` (prefix, for sortability)

These directly contradict each other.

### 3. Loading Mechanism is Unclear

The README.md says:
```
## Always Loaded
- default/dry_witted_engineering.md
- meta/project-index.md
- meta/PROVERBS.md
```

But "always loaded" has no technical meaning. Claude doesn't automatically read files. The actual loading happens via CLAUDE.md requirements:
```
- Claude MUST apply skills/default/dry_witted_engineering.md
- Claude MUST apply skills/meta/project-index.md
- Claude MUST read and internalize skills/meta/PROVERBS.md
```

**Problem:** This is imperative, not declarative. Claude must remember to read these files. After context compaction, Claude may forget. The README table and the CLAUDE.md requirements can drift apart.

### 4. Entry Point Inconsistency

Different skill clusters use different entry point patterns:

| Pattern | Used By |
|---------|---------|
| `SKILL.md` | terraform/, humanizer/, project/ |
| `README.md` | humanizer/ (alongside SKILL.md) |
| `INDEX.md` | github/ |
| `*-overview.md` | proto-*, doc-* |
| `*-index.md` | project-index.md |

**Problem:** When Claude looks for the entry point to a skill cluster, it has to guess the pattern. Wrong guesses waste context on files that reference other files without providing value.

### 5. The project/ vs meta/ Confusion

There are two "project" things:
1. `project/SKILL.md` - A slash command (`/project`) that initializes directories
2. `meta/project-*.md` - Methodology files about how to run projects

**Problem:** When someone says "load the project skill," which one? The CLAUDE.md says both:
- `Claude MUST apply skills/meta/project-index.md`
- The README table maps `/project` to `project/SKILL.md`

These serve different purposes but have the same name.

### 6. Proactive vs Reactive Loading Confusion

The README has a "Proactive Triggers" section:
```
| Condition | Load |
|-----------|------|
| Working in dotfiles, shell config | default/passive_qol.md |
| User mentions design critique | design/rigorous_critique.md |
```

**Problem:** "Proactive" loading requires Claude to:
1. Remember these conditions exist
2. Check conditions against current context
3. Decide to load the skill

After context compaction, Claude forgets the triggers exist. The "proactive" mechanism relies on Claude's memory, which is exactly what compaction destroys.

---

## Mistakes I Have Made

### 1. Reading Skills That Weren't Needed

When you ask about shell configuration, I've probably loaded `project-artifacts.md` (because CLAUDE.md says I must) even though DATA_SOURCES.md tracking is irrelevant to editing your `.zshrc`.

**Cost:** Token waste, diluted attention, potentially confusing instructions.

### 2. Not Reading Skills That Were Needed

When working on terraform configs in other projects, I may not have loaded `terraform/SKILL.md` because:
- The proactive trigger table isn't in my immediate context
- After compaction, I forgot it existed
- CLAUDE.md doesn't say "MUST apply terraform"

**Cost:** Reinventing patterns that are already documented.

### 3. Applying Stale Rules

The file versioning rules differ between CLAUDE.md (suffix) and project/SKILL.md (prefix). When I version files, I've probably been inconsistent about which pattern I use depending on which file I read most recently.

**Cost:** Inconsistent project structure.

### 4. Adding Content to CLAUDE.md Instead of Skills

When you told me "always do X," I added it to CLAUDE.md because that's where "always" rules go. But some of those rules are really process methodology (should be in a skill) or domain knowledge (should be in a domain skill).

**Cost:** CLAUDE.md grows without bound. Important rules get buried.

### 5. Missing the old/ Archive

There are 3 files in `old/` (protogen_stack.md, project_process.md, gradual_exploration_process.md) that contain the original monolithic versions of skills that were later split. I've never read these because:
- No README explains when to reference them
- They're marked "deprecated" so they seem irrelevant
- The loading instructions don't mention them

**Cost:** Lost context about why things were split the way they were. Potential patterns in the old files that didn't make it to the new ones.

---

## What "Better Structure" Would Enable

### 1. Clear Loading Semantics

Instead of "Claude MUST apply X" (imperative), declare a manifest:

```yaml
# .claude/manifest.yaml
always:
  - requirements.md    # Hard requirements (no emoji, permissions)
  - tone.md            # Communication style

on_context:
  terraform: terraform/SKILL.md
  proto: design/proto-overview.md
  documentation: documentation/doc-overview.md

on_project_init:
  - meta/project-index.md
  - meta/PROVERBS.md
```

This separates "what to load" from "the content." The manifest is the single source of truth for loading.

### 2. Single Source of Truth Per Rule

Every rule exists in exactly one place:
- Hard requirements go in `requirements.md`
- Process methodology goes in `meta/` skills
- Domain knowledge goes in domain skills

CLAUDE.md becomes thin: it references the manifest and contains only truly project-specific overrides for the dotfiles repo itself.

### 3. Consistent Entry Points

Every skill cluster uses the same pattern. Proposal: `SKILL.md` everywhere.

```
skills/
  terraform/SKILL.md     # Entry point
  proto/SKILL.md         # Entry point (currently proto-overview.md)
  documentation/SKILL.md # Entry point (currently doc-overview.md)
  meta/SKILL.md          # Entry point (currently project-index.md)
```

When Claude needs to load a skill, the pattern is always: `skills/<name>/SKILL.md`.

### 4. Separation of Concerns

```
.claude/
  manifest.yaml           # What to load when (single source)
  requirements.md         # Hard requirements (150 lines max)
  permissions.md          # What needs approval

  skills/
    tone/SKILL.md         # Communication style
    process/SKILL.md      # Project methodology (absorbs meta/)
    terraform/SKILL.md    # Domain: infrastructure
    proto/SKILL.md        # Domain: protobuf
    documentation/SKILL.md # Domain: writing
    ...
```

Benefits:
- `requirements.md` is short enough to always load
- Skills are only loaded when relevant
- Manifest is the single reference for what exists
- No duplication between locations

### 5. Deprecation Strategy

The `old/` directory should have a README explaining:
- Why each file was deprecated
- What it was split into
- When to reference it (e.g., "for historical context on why X pattern exists")

Or: absorb the useful parts into the new skills and delete the old files entirely. Half-measures (keeping files without explanation) create confusion.

---

## Questions for Design Decision

Before restructuring:

1. **What is the actual loading mechanism?** Does Claude Code have any built-in support for skill manifests, or is everything via CLAUDE.md instructions?

2. **Should CLAUDE.md be split?** If the loading mechanism is "Claude reads CLAUDE.md at start," then a 50-line CLAUDE.md that references other files may work better than a 350-line one.

3. **What about project-specific overrides?** Each project can have its own `.claude/CLAUDE.md`. Should skills be overridable per-project?

4. **Token budget reality:** How much context does "load everything" actually cost? Is the token waste from over-loading significant compared to the cost of missing context from under-loading?

5. **The humanizer/WARP.md question:** What is this file? It exists alongside SKILL.md with no explanation. Is it a variant? A mistake? Understanding this informs whether the current structure is intentional or accidental.

---

## Recommended Next Steps

1. **Audit for contradictions** - Systematically compare CLAUDE.md content against skill files to find all inconsistencies (the V2 suffix vs prefix example is likely not unique)

2. **Measure token cost** - Calculate actual token usage of "load everything" vs "load selectively" to quantify the waste

3. **Prototype manifest approach** - Try a minimal manifest.yaml to see if it actually helps Claude load the right things

4. **Consolidate duplicates** - Pick one location for each rule and delete the others

5. **Document old/** - Either write the README explaining when to reference old files, or delete them

---

## Summary

The current structure grew organically. Rules were added when problems were encountered. Skills were split when they got too large. But the loading mechanism, entry point conventions, and source-of-truth boundaries were never designed - they emerged.

The result: Claude's behavior depends on which files happen to be in context after the last compaction, not on a deliberate loading strategy. Better structure would make loading deterministic and context-appropriate.
