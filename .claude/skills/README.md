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
│   └── complete_developer_experience.md    # Tools + Docs + Agents completeness
├── communication/
├── meta/
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

## Usage

Skills are referenced in CLAUDE.md files:
- Global: `~/.claude/CLAUDE.md`
- Project: `<repo>/CLAUDE.md` or `<repo>/.claude/CLAUDE.md`

Reference with requirements block:
```markdown
# Requirements
- Claude MUST apply the skill defined in skills/default/dry_witted_engineering.md
```
