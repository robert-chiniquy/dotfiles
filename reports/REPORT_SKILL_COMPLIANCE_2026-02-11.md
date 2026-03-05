# Skill Compliance Report

Comparison of current `.claude/skills/` against Anthropic's "Complete Guide to Building Skills for Claude" (2026).

---

- [Source](#source)
- [Summary](#summary)
- [Structural Violations](#structural-violations)
- [Progressive Disclosure Violations](#progressive-disclosure-violations)
- [What the Guide Says](#what-the-guide-says)
- [Current Inventory and Status](#current-inventory-and-status)
- [Recommended Restructure](#recommended-restructure)
- [Special Cases](#special-cases)
- [Migration Notes](#migration-notes)

---

## Source

- Guide: https://resources.anthropic.com/hubfs/The-Complete-Guide-to-Building-Skill-for-Claude.pdf?hsLang=en
- Current skills: `/Users/rch/repo/dotfiles/.claude/skills/`

## Summary

Of 60 markdown files in `.claude/skills/`, **3 comply** with the guide's skill structure (terraform, humanizer, project). The remaining 57 are loose markdown files in category directories without YAML frontmatter or standard folder structure.

The guide defines a skill as:

```
your-skill-name/           # kebab-case folder
├── SKILL.md               # Required, with YAML frontmatter
├── scripts/               # Optional
├── references/            # Optional
└── assets/                # Optional
```

Current structure uses category directories (`default/`, `design/`, `meta/`) containing bare markdown files. This is not how skills work.

## Structural Violations

### 1. No SKILL.md entry point (57 files)

Every skill must have a file named exactly `SKILL.md`. Current files like `dry_witted_engineering.md`, `proto-overview.md`, `project-index.md` are not recognized as skills by Claude's skill loading system.

**Affected:** Everything except `terraform/SKILL.md`, `humanizer/SKILL.md`, `project/SKILL.md`

### 2. No YAML frontmatter (57 files)

The frontmatter is the most important part per the guide. It is always loaded into Claude's system prompt and determines when the skill body loads. Without it, Claude has no metadata to decide when to activate the skill.

Required minimum:
```yaml
---
name: your-skill-name
description: What it does. Use when user asks to [trigger phrases].
---
```

### 3. Category directories instead of skill folders

Current layout uses categories:
```
skills/
├── default/           # category, not a skill
│   ├── dry_witted_engineering.md
│   ├── passive_qol.md
│   └── bar_chart_comparison.md
├── design/            # category, not a skill
│   ├── systematic_feature_design.md
│   ├── proto-overview.md
│   └── ... 12 more files
```

Guide expects skill folders:
```
skills/
├── dry-witted-engineering/
│   └── SKILL.md
├── passive-qol/
│   └── SKILL.md
├── protogen/
│   ├── SKILL.md
│   └── references/
```

### 4. Folder naming violations

Guide requires kebab-case, no underscores, no capitals. Current violations:
- `dry_witted_engineering.md` (underscores)
- `bar_chart_comparison.md` (underscores)
- `passive_qol.md` (underscores)
- `check_feature_flag_conflicts.md` (underscores)
- `incomplete_work_audit.md` (underscores)
- `finding_uncommitted_work.md` (underscores)
- `jsonl_parsing.md` (underscores)
- `PROVERBS.md` (capitals)
- `MULTI-AGENT-COORDINATION-AMONG-SUBPROJECTS.md` (capitals)
- `INDEX.md` (capitals)

### 5. README.md inside skill folder

Guide explicitly says: "Don't include README.md inside your skill folder." Current violation: `humanizer/README.md`.

### 6. Root-level README.md as routing table

`skills/README.md` acts as a manual routing table ("User says X, load Y"). This is what YAML frontmatter descriptions are for. The routing table is a workaround for missing frontmatter.

## Progressive Disclosure Violations

The guide defines three levels:
1. **Frontmatter** - Always in system prompt, tells Claude when to use it
2. **SKILL.md body** - Loaded when skill triggers
3. **references/** - Loaded on demand within the skill

### Proto cluster (9 files, no progressive disclosure)

Currently 9 flat files in `design/`:
```
proto-overview.md, proto-schema.md, proto-architecture.md,
proto-database.md, proto-patterns.md, proto-project.md,
proto-testing.md, proto-frontend.md, proto-pitfalls.md
```

Should be:
```
protogen/
├── SKILL.md              # overview + when to load what
└── references/
    ├── schema.md
    ├── architecture.md
    ├── database.md
    ├── patterns.md
    ├── project.md
    ├── testing.md
    ├── frontend.md
    └── pitfalls.md
```

### Documentation cluster (12 files, same problem)

Currently 12 flat files in `documentation/`. Should be one skill with `references/`.

### Project process cluster (7 files in meta/)

Currently `project-index.md` plus 5 referenced files plus `PROVERBS.md`. Should be restructured into one or two skills with `references/`.

## What the Guide Says

Key requirements extracted from the guide:

| Requirement | Current State |
|-------------|--------------|
| SKILL.md exactly (case-sensitive) | 3 of 60 files comply |
| YAML frontmatter with `---` delimiters | 3 files have it |
| `name` field: kebab-case | 3 files comply |
| `description` includes WHAT + WHEN + triggers | 3 files comply |
| No XML tags in frontmatter | N/A (no frontmatter to violate) |
| Skill folder in kebab-case | 3 folders comply |
| No README.md in skill folder | 1 violation (humanizer/) |
| references/ for detail docs | Only terraform/ uses this |
| SKILL.md under 5,000 words | Most files are fine |
| Negative triggers where needed | Not present |

## Current Inventory and Status

### Compliant (3 skills)

| Skill | Status | Notes |
|-------|--------|-------|
| `terraform/SKILL.md` | PASS | Has frontmatter, references/, proper structure |
| `humanizer/SKILL.md` | PARTIAL | Has frontmatter, but has README.md (violation) |
| `project/SKILL.md` | PASS | Has frontmatter, slash command |

### Non-compliant standalone skills (need folder + frontmatter)

| Current File | Proposed Skill Name |
|-------------|-------------------|
| `default/dry_witted_engineering.md` | `dry-witted-engineering` |
| `default/passive_qol.md` | `passive-qol` |
| `default/bar_chart_comparison.md` | `bar-chart-comparison` |
| `default/casual_slack_tone.md` | `casual-slack-tone` |
| `default/technical_writing_voice.md` | `technical-writing-voice` |
| `design/systematic_feature_design.md` | `systematic-feature-design` |
| `design/socratic_discovery.md` | `socratic-discovery` |
| `design/rigorous_critique.md` | `rigorous-critique` |
| `design/pqthink.md` | `pqthink` |
| `design/complete_developer_experience.md` | `complete-developer-experience` |
| `engineering/structural_constraints.md` | `structural-constraints` |
| `engineering/check_feature_flag_conflicts.md` | `check-feature-flag-conflicts` |
| `codebase/incomplete_work_audit.md` | `incomplete-work-audit` |
| `codebase/finding_uncommitted_work.md` | `finding-uncommitted-work` |
| `utility/jsonl_parsing.md` | `jsonl-parsing` |
| `github/pr-status.md` | `pr-status` |

### Non-compliant clusters (need consolidation into skill + references/)

| Current Files | Proposed Skill | references/ contents |
|--------------|---------------|---------------------|
| `design/proto-*.md` (9 files) | `protogen` | 8 topic files |
| `documentation/doc-*.md` + 5 others (12 files) | `documentation` | 11 topic files |
| `meta/project-*.md` + PROVERBS (7 files) | `project-process` | 6 topic files |

### Special files (not skills)

| File | Disposition |
|------|------------|
| `skills/README.md` | Delete after migration (replaced by frontmatter) |
| `humanizer/README.md` | Delete (violates guide) |
| `humanizer/WARP.md` | Move to `humanizer/references/warp.md` |
| `github/INDEX.md` | Absorb into `pr-status/SKILL.md` |
| `old/*.md` (3 files) | Keep as archive, not skills |

## Recommended Restructure

### Target layout

```
skills/
├── bar-chart-comparison/
│   └── SKILL.md
├── casual-slack-tone/
│   └── SKILL.md
├── check-feature-flag-conflicts/
│   └── SKILL.md
├── complete-developer-experience/
│   └── SKILL.md
├── documentation/
│   ├── SKILL.md
│   └── references/
│       ├── process.md
│       ├── content.md
│       ├── templates.md
│       ├── verify.md
│       ├── learnings.md
│       ├── organization.md
│       ├── rap.md
│       ├── layered.md
│       ├── merging.md
│       ├── tone-matrixing.md
│       └── marketing-lens.md
├── dry-witted-engineering/
│   └── SKILL.md
├── finding-uncommitted-work/
│   └── SKILL.md
├── humanizer/
│   ├── SKILL.md
│   └── references/
│       └── warp.md
├── incomplete-work-audit/
│   └── SKILL.md
├── jsonl-parsing/
│   └── SKILL.md
├── multi-agent-coordination/
│   └── SKILL.md
├── passive-qol/
│   └── SKILL.md
├── pqthink/
│   └── SKILL.md
├── pr-status/
│   └── SKILL.md
├── project/
│   └── SKILL.md
├── project-process/
│   ├── SKILL.md
│   └── references/
│       ├── artifacts.md
│       ├── practices.md
│       ├── organization.md
│       ├── priorities.md
│       ├── multisubproject.md
│       └── proverbs.md
├── protogen/
│   ├── SKILL.md
│   └── references/
│       ├── schema.md
│       ├── architecture.md
│       ├── database.md
│       ├── patterns.md
│       ├── project.md
│       ├── testing.md
│       ├── frontend.md
│       └── pitfalls.md
├── rigorous-critique/
│   └── SKILL.md
├── socratic-discovery/
│   └── SKILL.md
├── structural-constraints/
│   └── SKILL.md
├── systematic-feature-design/
│   └── SKILL.md
├── technical-writing-voice/
│   └── SKILL.md
├── terraform/
│   ├── SKILL.md
│   └── references/ (exists)
└── old/ (archive, not skills)
```

**Count: 25 skill folders** (down from 60 loose files across 12 category directories)

### Example frontmatter for each type

**Always-active skill** (broad description so it triggers on everything):
```yaml
---
name: dry-witted-engineering
description: |
  Default communication tone for all engineering work. Active for all
  technical tasks including code review, design discussion, commit messages,
  documentation, and status reports. Use when any engineering output is
  being produced.
---
```

**Context-triggered skill:**
```yaml
---
name: passive-qol
description: |
  Proactive quality-of-life suggestions for computing environment. Use when
  working in dotfiles, shell config, system configuration, or when user
  mentions friction, annoyance, or inefficiency in their setup.
---
```

**Domain skill with references:**
```yaml
---
name: protogen
description: |
  Protobuf and gRPC architecture patterns for the protogen stack. Use when
  user mentions "proto", "protobuf", "grpc", "codegen", or works with .proto
  files. Covers schema design, three-layer architecture, database mapping,
  and common pitfalls.
metadata:
  version: 2.0.0
---
```

**MCP-enhanced skill (future):**
```yaml
---
name: connector-dev
description: |
  Connector development workflow using baton-sdk. Use when building or
  debugging connectors, working with c1z files, or running sync operations.
  Coordinates with connector MCP tools for inspection and validation.
metadata:
  mcp-server: baton-connector
  version: 1.0.0
---
```

## Special Cases

### PROVERBS.md

Not a skill in the traditional sense - it's guiding principles. Two options:

**Option A:** Make it a reference file inside `project-process/references/proverbs.md`. The project-process SKILL.md body says "consult references/proverbs.md for guiding principles." CLAUDE.md continues to say "read and internalize proverbs."

**Option B:** Keep a separate `proverbs/SKILL.md` with very broad triggers so it loads on everything. The description says "Guiding principles for all work."

Recommend **Option A** - proverbs are part of the project process methodology, not a standalone skill.

### CLAUDE.md "always loaded" references

Current CLAUDE.md says:
```
- Claude MUST apply skills/meta/project-index.md
- Claude MUST read and internalize skills/meta/PROVERBS.md
```

After migration, update to:
```
- Claude MUST apply skills/project-process/SKILL.md
```

The frontmatter descriptions for "always-active" skills should be broad enough that they trigger automatically. CLAUDE.md becomes the backstop, not the primary loading mechanism.

### skills/README.md routing table

Currently acts as a manual routing table. After migration, the YAML frontmatter descriptions on each skill serve this purpose. The README.md can be deleted or reduced to a human-readable catalog (not for Claude's use).

## Migration Notes

### Risk: Symlinked directory

`.claude/` is symlinked from `$HOME` to the dotfiles repo. All changes must happen in the repo. Test that skill loading works after restructure by checking `/mcp` output or asking Claude "what skills are available?"

### Risk: Breaking existing CLAUDE.md references

CLAUDE.md references specific file paths like `skills/default/dry_witted_engineering.md`. All references must be updated after restructure.

### Risk: old/ directory

Move current category directories to `old/` before creating new structure. This preserves history per the established deprecation pattern.

### Execution order

1. Create new skill folders with SKILL.md + frontmatter
2. Move reference content into references/
3. Update CLAUDE.md to reference new paths
4. Move old category directories to old/
5. Delete skills/README.md routing table
6. Test triggering for each skill
