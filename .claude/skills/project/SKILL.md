---
name: project
version: 1.5.0
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

- `scripts/` - Reusable analysis scripts (batch operations, single approval)
- `reports/` - Analysis outputs (created on demand)
- `data/` - Collected data files (created on demand)

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

### Script-Based Analysis

When research requires many shell commands:
1. Write scripts to `scripts/` - NOT individual commands
2. Make scripts repeatable and idempotent
3. One approval to run the script, not 100s of commands
4. Output to `reports/` or `data/`
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

## Public vs Private Projects

**Every project is either public or private. This distinction is critical.**

### Private Projects

Private projects may contain privileged information:
- PII (names, emails, user data)
- IDs (internal identifiers, account numbers)
- Passwords and credentials
- Cryptographic secrets (keys, tokens)
- Internal URLs, repo paths, tool names
- Proprietary business logic

**The sources of a private project may themselves be private** (e.g., a secret gist, internal wiki, private repo). Never expose these sources in any output that could become public.

### Public Projects

Public projects contain only information safe for external visibility:
- Open source code
- Public documentation
- Generic examples
- Anonymized data

### Publishing Flow

There may be a regular flow of sanitized data or code from private to public projects. This is always:
1. **Explicitly set up** - never automatic
2. **Sanitized** - all privileged information removed
3. **Reviewed** - human approval before publishing

### Export Tracking in project.md

When a private project publishes to a public project, track it in `project.md`:

```markdown
## Exports

**Visibility:** Private
**Publishes to:** /path/to/public/project (or repo URL)

### Whitelist (allowed to export)
- `axioms/` - Formal specifications (sanitized)
- `README.md` - Public documentation
- `Makefile` - Build instructions

### Blacklist (never export)
- `data/` - Contains internal identifiers
- `reports/` - Contains private analysis
- `scripts/` - References internal paths
- `DATA_SOURCES.md` - Lists private sources
```

Exports must be whitelisted. Common blacklist items:
- `./data/` - Often contains raw, unsanitized data
- `./reports/` - May contain internal analysis with private references
- `./scripts/` - May hardcode internal paths or credentials
- Project meta-files (DATA_SOURCES.md, LEARNINGS.md, project.md)

### When Data Sources Are Private

If you encounter a private project among your data sources:
1. **Ask what to do** - don't assume
2. **Never reference it in public outputs** - no paths, URLs, or identifiers
3. **Anonymize any derived information** - change names, IDs, specifics
4. **Document the boundary** - note in DATA_SOURCES.md that certain sources are private

### Indicators of Private Projects

- Located in private repos or internal systems
- Contains `/Users/`, internal domains, or proprietary tool names
- References specific people, customers, or accounts
- Contains API keys, tokens, or credentials
- Marked as confidential or internal

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

### Script-Based Analysis (Research Projects)

When research requires many shell commands (traversing repos, pattern matching, data collection):

1. **Write scripts to `scripts/`** - NOT individual commands
2. **Make scripts repeatable** - idempotent, re-runnable, self-contained
3. **One approval** - run the whole script, not 100s of individual commands
4. **Output to files** - results go to `reports/` or `data/`, not inline

Individual command approval does not scale. Batch operations into scripts.

### Goals and Topics Tracking

The `project.md` file (if present) should maintain a running list of all:
- Goals stated or implied throughout the project
- Outcomes achieved or desired
- Deliverables requested or produced
- Topics raised or explored

This list grows throughout the project - never remove items. Mark completed items with status. This creates a complete record of project scope evolution and prevents goals from being forgotten across sessions.

### Visualization Rule: Dotfiles to SVG

When creating Graphviz `.dot` files:
1. **Always render to SVG** - dotfiles are source, SVGs are output
2. **Add Makefile target** - every dotfile gets a corresponding `make` target
3. **Output to same directory** - `foo.dot` → `foo.svg` in same location

Example Makefile pattern:
```makefile
DOTS := $(wildcard axioms/visualizations/*.dot)
SVGS := $(DOTS:.dot=.svg)

.PHONY: viz
viz: $(SVGS) ## Render all dotfiles to SVG

%.svg: %.dot
	dot -Tsvg $< -o $@
```

4. **Embed in README** - SVGs should be included in README.md with explanations:
```markdown
### Diagram Title

![Description](path/to/diagram.svg)

**What this shows**: Explanation of what the visualization communicates...
```

This ensures visualizations are always viewable without requiring graphviz installation, and that readers understand what they're looking at.

### No Time-Based Critiques

Time estimates are forbidden by global rules. Corollary: **critiques cannot be based on time comparisons.**

Invalid critique patterns:
- "This takes 40 hours but should take 8 hours" - you don't know either number
- "Reduce scope to save time" - scope decisions are about value, not time
- "Original: 32h, Recommended: 8h" - fabricated numbers dressed as analysis

Valid critique patterns:
- "This is over-engineered because X feature provides no value"
- "Reduce scope because Y component solves a problem that doesn't exist"
- "Simplify because Z approach requires maintaining code nobody will use"

Critique based on **value and necessity**, not imagined effort comparisons.

### Upstream/Downstream Project Graph

Upstream projects (planning, design, meta-analysis) should maintain a visual graph showing:
- Which data sources feed into the project
- Which subprojects exist and their relationships
- Which downstream projects receive outputs

Place at the top of the project README as embedded SVG:

```markdown
# Project Name

![Project graph](project_graph.svg)

## Overview
...
```

The graph should be Graphviz source in `project_graph.dot` with a Makefile target to render it. Update when:
- New data sources are added
- New subprojects emerge
- New downstream consumers are identified

This is meta-documentation that helps orient anyone entering the project.

### Upstream/Downstream Agent Coordination

When an upstream agent (working on higher-level planning or design) has knowledge of a downstream agent concerned with specific topic areas:

1. **Read the downstream agent's work** - Before planning, review:
   - The downstream project's goals and scope
   - Existing approaches and implementations
   - DATA_SOURCES.md for what they've already researched
   - LEARNINGS.md for discoveries that inform upstream decisions

2. **Coordinate, don't duplicate** - Upstream work should:
   - Reference downstream approaches rather than reinvent
   - Note dependencies on downstream deliverables
   - Flag conflicts between upstream design and downstream implementation

3. **Document the relationship** - In DATA_SOURCES.md:
   ```markdown
   ## Related Projects
   - `/path/to/downstream/project` - Implementing X (downstream)
   ```

4. **Respect convergence** - If a downstream project started independently and later became relevant to upstream work, treat its existing approaches as constraints unless explicitly told to override.

This ensures higher-level planning (upstream) incorporates ground-truth from implementation work (downstream), and that effort isn't wasted on incompatible approaches.
