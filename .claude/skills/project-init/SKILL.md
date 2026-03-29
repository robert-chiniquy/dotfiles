---
name: project-init
disable-model-invocation: true
description: |
  Initialize a directory with the project framework. Creates DATA_SOURCES.md,
  LEARNINGS.md, GLOSSARY.md, .claude/CLAUDE.md, .envrc with accent color.
  Usage: /project-init [topic]
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

1. **DATA_SOURCES.md** — empty template with header
2. **LEARNINGS.md** — empty with `## YYYY-MM-DD HH:MM: [topic]` format note
3. **GLOSSARY.md** — empty with header
4. **.claude/CLAUDE.md** — project-specific instructions (document index, build commands, key context)
5. **.envrc** — `export PROMPT_ACCENT="#color"` (pick from vaporwave palette based on project character)

## If Existing Codebase

Add to `.claude/CLAUDE.md`:
```
These meta-documents are local-only and will not be committed.
```

Add to `.gitignore`:
```
DATA_SOURCES.md
LEARNINGS.md
GLOSSARY.md
FAILURES.md
```

## Accent Color Palette

Pick based on project character:
* `#5cecff` (cyan) — infrastructure, tooling
* `#ff0099` (hot pink) — user-facing, frontend
* `#fbb725` (gold) — data, analytics
* `#aa00e8` (purple) — experimental, research
* `#ff00f8` (magenta) — integration, connectors
