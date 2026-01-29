---
name: project
version: 1.4.0
description: |
  Initialize current directory with the global project process framework.
  Creates DATA_SOURCES.md, LEARNINGS.md, GLOSSARY.md, .claude/CLAUDE.md,
  .envrc, and Makefile. Usage: /project [topic] where topic describes
  the project focus (e.g., /project research-ducks).
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - AskUserQuestion
---

# Project Initialization

Initialize the current working directory with the global project process framework.

## Usage

```
/project [topic]
```

- `topic` (optional): Brief description of project focus
- Project name is derived from directory name
- If topic not provided, ask for one

## Process

### Step 1: Determine Project Name and Topic

1. Project name = current directory name (e.g., `ducks`)
2. Topic = first argument if provided (e.g., `research-ducks`)
3. If no topic provided, ask: "What is this project about?"

### Step 2: Ensure Initial Data Source

**MANDATORY: Every project must have at least one data source from the start.**

1. Check for existing .md files: `ls *.md 2>/dev/null`
2. If .md files exist, scan them for data sources (URLs, file paths, references)
3. If no existing data sources found, ask: "What is the first data source? (URL, file path, or reference)"

Do NOT proceed to file creation without at least one data source. A project without a data source has no grounding.

### Step 3: Choose Accent Color

Select from vaporwave palette based on topic keywords:

| Keywords | Color | Hex |
|----------|-------|-----|
| infra, deploy, ci, ops, k8s, terraform | cyan | `#5cecff` |
| user, ui, frontend, web, app | hot pink | `#ff0099` |
| data, analytics, metrics, stats | gold | `#fbb725` |
| research, experiment, poc, spike | purple | `#aa00e8` |
| other/default | magenta | `#ff00f8` |

### Step 4: Create Files

**Only create these files. Other artifacts created on-demand.**

#### DATA_SOURCES.md

```markdown
# Data Sources

Track provenance of all information. Add sources as consulted, not retroactively.

## Filesystem

## URLs

## Other
```

**Populate with the initial data source from Step 2.** Place it under the appropriate section (Filesystem for paths, URLs for links, Other for everything else).

#### LEARNINGS.md

```markdown
# Learnings

Preserve discoveries with dated headers. Append-only.
```

#### GLOSSARY.md

```markdown
# Glossary

Domain-specific terminology for this project.

| Term | Definition |
|------|------------|
```

#### Makefile

```makefile
# {PROJECT_NAME}
#
# Self-documenting Makefile. Run `make` or `make help` to see available targets.

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
```

#### .envrc

```bash
export PROMPT_ACCENT="{CHOSEN_COLOR}"
```

#### .claude/CLAUDE.md

```markdown
# CLAUDE.md

## {PROJECT_NAME}

{TOPIC}

## Build

Run `make help` to see available targets.

## Project Structure

Every project has a research phase. Not every project has a deliverable.

**Phase:** Research | Design | Implementation | Complete

<!-- Update phase as work progresses -->

## Subprojects

<!-- List subprojects if any, each with their own structure -->

## Architecture

<!-- Key directories and their purposes -->

## Conventions

### File Organization

When files of a type become numerous (>5-10), move to a directory:
- **By type**: Many of one kind -> `plans/`, `reports/`, `designs/`
- **By version**: Many versions of many things -> `v1/`, `v2/` or `old/`

Choose based on what makes retrieval easier.

### Versioning

- **Documents**: Feedback = new version (`_V2.md`), not overwrite
- **Identifiers**: Numbers are permanent, removed items get ~~strikethrough~~
- **Plans**: `PLAN_<OBJECTIVE>.md`, multiple can coexist
- **Commits**: Checkpoint at phase boundaries and before major changes
```

### Step 5: Final Setup

1. Run `direnv allow` if direnv available
2. Optionally initialize git if not already a repo

## Output

After completion, summarize:

```
Project "{PROJECT_NAME}" initialized
Accent: {COLOR_NAME} ({HEX})
Phase: Research
Initial data source: {DATA_SOURCE}

Created:
  DATA_SOURCES.md (with initial source)
  LEARNINGS.md
  GLOSSARY.md
  Makefile
  .envrc
  .claude/CLAUDE.md
```

## What This Skill Does NOT Create

Created on-demand during project work:

- `old/` - When code is deprecated
- `scripts/` - When scripts are written
- `plans/` - When plans become numerous
- `reports/` - When reports become numerous
- `HUMAN_TODOS.md` - When blocking action occurs
- `PLAN_*.md` - When entering plan mode
- `FAILURES.md` - When failures are documented
- `DEMO.md` - When user-facing features complete
- Subproject directories - When subprojects emerge

## Project Types

### Research-only
- Goal: Understand something
- Deliverable: LEARNINGS.md, possibly a report

### Research-to-deliverable
- Starts as research spike
- May produce: code, design doc, recommendation
- Transition: When research reveals what to build

### Implementation
- Clear deliverable from start
- Research phase: Understand constraints, prior art
- Build phase: Write code, tests, docs

## Research Depth

When spiking on research, clarify depth if unclear:

| Depth | Output |
|-------|--------|
| Shallow | "Can this work?" Yes/no |
| Medium | "How would this work?" Design sketch |
| Deep | "Make this work." Working prototype |

## Key Practices Reference

### Stable Identifiers
Numbers are forever. Gaps are fine. ~~Strikethrough~~ removed items.

### Subjunctive = Exploratory
"perhaps", "could", "might" = do it if possible. Only skip if technically impossible.

### Show Me the Code
Claims cite files, line numbers, or commits. Don't theorize - verify.

### Show Me the Source
When quoting documentation in summaries or reports, include the URL. Every quote needs a link. This is non-negotiable - a quote without a URL is unverifiable and therefore useless.

### Version, Don't Overwrite
When receiving feedback on a document, NEVER edit it in place. Create a new version with **prefix** (for sortability):
- `V2_FILENAME.md`, `V3_FILENAME.md`
- `REVISED_FILENAME.md`, `FINAL_FILENAME.md`
- `PHASE2_FILENAME.md`

Prefix, not suffix. `V2_FOO.md` sorts next to `V1_FOO.md`. `FOO_V2.md` does not.

The only exception: typo fixes or minor corrections that don't change meaning.

Feedback includes: corrections, additions, "you forgot X", "add Y", "fix Z". If the user points out something wrong or missing, that's feedback - version the file.

### Catalogs
Enumerable things get a catalog .md. Catalog is authoritative.
