# Claude Skills

Personal skill definitions for Claude Code.

## Structure

```
skills/
├── default/
│   └── dry_witted_engineering.md   # Default communication style
├── engineering/
│   └── structural_constraints.md   # Architectural patterns for compile-time safety
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

## Usage

Skills are referenced in CLAUDE.md files:
- Global: `~/.claude/CLAUDE.md`
- Project: `<repo>/CLAUDE.md` or `<repo>/.claude/CLAUDE.md`

Reference with requirements block:
```markdown
# Requirements
- Claude MUST apply the skill defined in skills/default/dry_witted_engineering.md
```
