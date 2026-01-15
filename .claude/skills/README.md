# Claude Skills

Personal skill definitions for Claude Code.

## Structure

```
skills/
├── default/
│   └── dry_witted_engineering.md   # Default communication style
├── engineering/
│   └── structural_constraints.md   # Architectural patterns for compile-time safety
├── design/
│   ├── systematic_feature_design.md        # 10-step design process with Level Framework
│   ├── socratic_discovery.md               # Question-driven requirements discovery
│   ├── rigorous_critique.md                # Three-lens design review
│   ├── complete_developer_experience.md    # Tools + Docs + Agents completeness
│   └── protogen_stack.md                   # Proto-first architecture patterns
├── documentation/
│   ├── layered_documentation.md          # Three-layer documentation strategy
│   └── gradual_exploration_process.md    # 7-step content pattern, 8-step process
├── communication/
├── meta/
│   └── project_process.md          # Mandatory project practices (DATA_SOURCES.md, etc.)
└── README.md
```

## Skills

### default/dry_witted_engineering.md
Default communication mode for all engineering work. Optimizes for technical correctness, salience, and restraint. Applied automatically unless overridden.

### engineering/structural_constraints.md
Architectural philosophy for making mistakes structurally impossible rather than relying on developer discipline. Covers:
- Scoped context propagation
- Explicit naming of dangerous operations
- Single source of truth via code generation
- Fail-closed defaults
- Interface segregation for minimal authority

### design/systematic_feature_design.md
10-step methodology for designing features: Research → Compare → Refine → Ideate → Discover → Implement → Critique → Double Back → Consolidate → Document. Includes Level Framework (0=Platform, 1=Workflow, 2=Polish) to ensure fundamentals before polish.

### design/socratic_discovery.md
Use progressively revealing questions to build consensus rather than asserting requirements. 7-step arc: Empathy → Tension → Exposure → Challenge → Comparison → Evaluation → Value. Leads stakeholders to discover problems themselves.

### design/rigorous_critique.md
Three-lens design review before implementation: (1) Unnecessary Complexity - identify over-engineering, (2) Missing Fundamentals - check Level 0 coverage, (3) Feasibility and Value - assess benefit vs cost. Expected outcome: 30-50% feature reduction.

### design/complete_developer_experience.md
Ensure all three components of DX: Tools (CLI, servers, build systems), Documentation (ontology, progressive disclosure, examples), Agents (AI assistance, code generation). Don't ship with only one leg. Minimum: tools + docs. Add agents post-launch for delight.

### design/protogen_stack.md
Proto-first architecture for Go backends with TypeScript frontends. Covers proto organization, code generation, three-layer architecture, Driver/Controller pattern, WithPassport tenant isolation, FormSchema for dynamic UIs, and frontend patterns (@protobuf-ts, WebSocket).

### documentation/layered_documentation.md
Three-layer strategy for documentation: (1) Global Skills - reusable patterns, (2) Local Implementation Notes - how this project applies patterns, (3) Domain Design - project-specific models and logic. Prevents conflation of stack knowledge with domain knowledge.

### documentation/gradual_exploration_process.md
Systematic process for writing technical documentation. Core concepts:
- **7-step content pattern**: Orient, Contextualize, Explain, Tradeoffs, Demonstrate, Edge Cases, Connect
- **8-step process**: Pre-flight, Scope, Source, Outline, Draft, Verify, Contradiction, Connect, Publish
- **Phases**: Iterative passes that incorporate new learnings
- **Levels**: L0 (critical) before L1 (important) before L2 (supporting) before L3 (reference)
- **Hard-won learnings**: Avoid version numbers, code is ground truth, document anti-patterns, assertion verification before publication

### meta/project_process.md
Mandatory practices for all projects. Non-negotiable requirements:

**Mandatory Artifacts:**
- **DATA_SOURCES.md** - Track provenance of all information: filesystem paths, URLs, other sources. Add as consulted, not retroactively.
- **LEARNINGS.md** - Preserve discoveries with dated headers (## YYYY-MM-DD: Topic). Append-only, include file paths and code snippets.
- **HUMAN_ACTIONS_NEEDED.md** - Queue blocking human actions instead of stopping. Continue other work while waiting.
- **old/** directory - Move deprecated code here instead of deleting. Each item gets README explaining why deprecated and what replaced it.
- **DEMO.md** - After completing user-facing features, write runnable walkthrough with expected output.

**Mandatory Practices:**
- **File versioning** - Create V2, V3, PHASE2 versions instead of overwriting documents (except typo fixes).
- **Stable identifiers** - Never renumber backlog items. Keep holes in sequence, strikethrough removed items.
- **Checkpoint commits** - Commit at end of each phase, before major changes, when meaningful work completes.
- **Catalog documents** - Maintain markdown catalogs for enumerable items (components, endpoints, etc.).

## Usage

Skills are referenced in CLAUDE.md files:
- Global: `~/.claude/CLAUDE.md`
- Project: `<repo>/CLAUDE.md` or `<repo>/.claude/CLAUDE.md`

Reference with requirements block:
```markdown
# Requirements
- Claude MUST apply the skill defined in skills/default/dry_witted_engineering.md
```
