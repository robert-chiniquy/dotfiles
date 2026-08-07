---
name: project-init
disable-model-invocation: true
description: |
  Initialize a directory with the project framework. Creates LEARNINGS.md,
  .claude/CLAUDE.md, .envrc with accent color; ensures README has a Sources
  section. Usage: /project-init [topic]
argument-hint: "[topic]"
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - AskUserQuestion
---

# Project Init

Initialize `$ARGUMENTS` as a project. If no argument, use the current directory's purpose.

## Create These Files

1. **LEARNINGS.md** — empty with `## YYYY-MM-DD HH:MM: [topic]` format note
2. **.claude/CLAUDE.md** — project-specific instructions (document index, build commands, key context)
3. **.envrc** — `export PROMPT_ACCENT="#color"` (pick from vaporwave palette based on project character)
4. **README.md** — create or update with a `## Sources` section for provenance (no DATA_SOURCES.md, no GLOSSARY.md)

## If Existing Codebase

Add to `.claude/CLAUDE.md`:
```
These meta-documents are local-only and will not be committed.
```

Add to `.gitignore`:
```
LEARNINGS.md
FAILURES.md
PLAN_*.md
```

## Accent Color Palette

Pick based on project character:
* `#5cecff` (cyan) — infrastructure, tooling
* `#ff0099` (hot pink) — user-facing, frontend
* `#fbb725` (gold) — data, analytics
* `#aa00e8` (purple) — experimental, research
* `#ff00f8` (magenta) — integration, connectors
