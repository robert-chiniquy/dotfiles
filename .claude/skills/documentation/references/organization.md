# Documentation Organization

Project structure and file naming for documentation.

## Directory Structure

```
project/
├── docs/                    # Publishable (best-of-breed)
│   ├── 00_INDEX.md
│   ├── 01_GETTING_STARTED.md
│   ├── 02_CORE_CONCEPTS.md
│   └── ...
├── PHASE_1/                 # Superseded Phase 1 docs
├── PHASE_2/                 # Superseded Phase 2 docs
├── CLAUDE.md                # Project instructions
├── DOCUMENTATION_PROCESS.md # Process docs (internal)
├── *_INVESTIGATION.md       # Research findings (internal)
└── *_RESEARCH.md            # Research notes (internal)
```

Principles:
- `docs/` contains only publishable content
- Phase folders archive superseded work
- Root contains internal/research files

## File Naming

Publishable docs (in docs/):
```
NN_SECTION_NAME.md
```

- `NN` = 2-digit sequence (01-99)
- `SECTION_NAME` = SCREAMING_SNAKE_CASE
- Numbers correspond to ontology position
- Gaps reserved for future sections

Examples:
```
01_GETTING_STARTED.md
02_CORE_CONCEPTS.md
03_BUILDING_CONNECTORS.md
06_COOKBOOK.md
10_GLOSSARY.md
```

Internal docs (at root):
```
TOPIC_PHASE_N.md
TOPIC_INVESTIGATION.md
TOPIC_RESEARCH.md
```

## Ontology Index

Create `00_INDEX.md` in docs/ that:
1. Maps numbered files to ontology sections
2. Shows full ontology structure
3. Indicates existing vs reserved sections
4. Provides reading paths by persona
5. Documents naming convention

## When to Create Phase Folders

Create `PHASE_N/` when:
- Document superseded by MERGED or V2 version
- Preserve earlier thinking without cluttering workspace
- Multiple iteration phases produced distinct versions

Don't move to phase folder:
- Research documents
- Process documents
- Investigation findings

## Version Progression

```
TOPIC_PHASE_1.md           # Initial draft
    |
    v (feedback, new info)
TOPIC_PHASE_2.md           # Second pass
    |
    v (merge sources)
TOPIC_MERGED.md            # Consolidated
    |
    v (move to docs/, rename)
docs/NN_TOPIC.md           # Publishable

# Archive superseded:
PHASE_1/TOPIC_PHASE_1.md
PHASE_1/TOPIC_PHASE_2.md
```
