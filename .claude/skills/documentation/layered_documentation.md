# Documentation Skill: Layered Documentation Strategy

## Purpose

Define clear boundaries between different types of documentation to prevent conflation of reusable patterns with project-specific decisions.

## When to Apply

- Starting a new project that uses established patterns (e.g., protogen stack)
- Creating documentation that might be reused across projects
- Reviewing existing docs to identify misplaced content
- Onboarding others to a codebase

---

## The Three Layers

### Layer 1: Global Skills/Patterns

**Location**: `~/.claude/skills/` or equivalent shared location

**Contains**:
- Reusable architectural patterns
- Technology stack conventions
- Design methodologies
- Coding standards that apply across projects

**Characteristics**:
- Project-agnostic
- Stable over time
- Applicable to multiple codebases
- Focused on "how to build things"

**Examples**:
- Protogen Stack architecture pattern
- Driver/Controller interface pattern
- FormSchema as A2UI building block
- Three-layer architecture (RPC → Controller → DB)
- Frontend stack conventions (Next.js, MUI, @protobuf-ts)

**Test**: "Would this be useful to someone starting a completely different project?"

---

### Layer 2: Local Implementation Notes

**Location**: `<project>/PROTOGEN_STACK.md`, `<project>/ARCHITECTURE.md`, or similar

**Contains**:
- How this project applies global patterns
- Which stack components are used
- Lineage/derivation from reference implementations
- Deviations from standard patterns and why
- Project-specific configuration

**Characteristics**:
- References global patterns, doesn't redefine them
- Documents choices made for this project
- Explains simplifications or extensions
- Links to source patterns

**Examples**:
- "We use c1's Form pattern but removed FieldGroup"
- "Storage is abstracted but we only implement JSONFile for now"
- "No tenant isolation - single-user experimentation"
- "Proto organization: v1 for TUI, v2 for web"

**Test**: "Does this explain how we're using a known pattern, not define the pattern itself?"

---

### Layer 3: Domain-Specific Design

**Location**: `<project>/DESIGN.md`, `<project>/DESIGN_<feature>.md`

**Contains**:
- Domain model (entities, relationships)
- Business logic and semantics
- Feature-specific decisions
- API contracts for this application
- UI/UX design

**Characteristics**:
- Specific to the problem being solved
- Uses patterns from Layer 1, configured per Layer 2
- Would not apply to a different project
- Focused on "what we're building"

**Examples**:
- Session, Cell, Workspace model
- Re-execution semantics (immutable history, branching)
- ClarifyingQuestion as agent-to-user interaction
- Artifact types (SVG, TABLE, CANVAS)
- UI layout (notebook column, workspace column)

**Test**: "Is this about the problem domain, not about how we build software?"

---

## Decision Tree

When writing documentation, ask:

```
Is this a reusable pattern applicable to other projects?
├── Yes → Layer 1 (Global Skill)
└── No
    ├── Is this about how we apply a known pattern?
    │   ├── Yes → Layer 2 (Local Implementation Notes)
    │   └── No → Layer 3 (Domain-Specific Design)
```

---

## Anti-Patterns

### 1. Pattern Leakage into Design Docs

**Wrong**: DESIGN.md explains what the Driver/Controller pattern is

**Right**: DESIGN.md uses Driver/Controller without explaining it; references PROTOGEN_STACK.md if needed

### 2. Domain Logic in Stack Docs

**Wrong**: PROTOGEN_STACK.md describes Session/Cell model

**Right**: PROTOGEN_STACK.md describes how we organize protos; DESIGN.md describes what's in them

### 3. Re-documenting Global Patterns Locally

**Wrong**: Local ARCHITECTURE.md re-explains three-layer architecture

**Right**: Local doc says "We follow three-layer architecture (see global skill)" and documents deviations

### 4. Implementation Details in Global Skills

**Wrong**: Global skill includes "we use KSUID for session IDs"

**Right**: Global skill says "use time-sortable IDs (e.g., KSUID)"; local doc says "we use KSUID"

---

## File Naming Conventions

| Layer | Naming Pattern | Examples |
|-------|----------------|----------|
| Global Skills | `<topic>.md` in skills dir | `protogen_stack.md`, `structural_constraints.md` |
| Local Implementation | `<PATTERN>_STACK.md`, `ARCHITECTURE.md` | `PROTOGEN_STACK.md` |
| Domain Design | `DESIGN.md`, `DESIGN_<feature>.md` | `DESIGN.md`, `DESIGN_WEB.md` |

---

## Public vs Private Repository Awareness

When documenting patterns derived from other codebases, observe repository visibility:

### Private Repositories

**Examples**: `~/repo/c1`, proprietary internal codebases

**Rules**:
- NEVER reference by name or path in global skills
- NEVER include proprietary patterns without abstraction
- Use generic descriptions: "production codebase", "reference implementation"
- Extract the *pattern*, not the *implementation details*
- When uncertain, ask the user

**In global skills**:
```markdown
// WRONG
Derived from c1's form pattern in ~/repo/c1/protos/c1api/c1/api/form/v1/form.proto

// RIGHT  
This pattern is common in production proto-first applications for dynamic form generation.
```

**In local project docs** (private repo):
```markdown
// OK - local docs can reference private repos the user has access to
Derived from c1's Form pattern: c1/api/form/v1/form.proto
```

### Public Repositories

**Examples**: Open source projects, public GitHub repos

**Rules**:
- May reference by name and URL in global skills
- Can link to specific files/patterns
- Still prefer abstracting the pattern over copying verbatim

### Decision Flow

```
Is the source repository public?
├── Yes → May reference in global skills (prefer abstraction)
├── No (private) → 
│   ├── Global skills: Abstract the pattern, no direct references
│   └── Local docs: May reference if user has access
└── Uncertain → Ask the user before documenting
```

### Why This Matters

- Global skills may be shared or synced across machines
- Private repo paths expose proprietary information
- Patterns are valuable; implementation details may be confidential
- Local project docs are scoped to users with repo access

---

## Maintenance

### When to Update Each Layer

**Global Skills**: When you discover a pattern that would help future projects
- After completing a project, extract reusable learnings
- When you find yourself re-explaining something across projects

**Local Implementation**: When project configuration changes
- Adding/removing stack components
- Changing how you apply a pattern
- Documenting new deviations

**Domain Design**: When requirements or design changes
- New features
- Changed semantics
- Revised data model

### Cross-References

- Domain docs MAY reference local implementation notes
- Local implementation notes SHOULD reference global skills
- Global skills SHOULD NOT reference specific projects (except as examples)

---

## Example: Agent Notebook Project

```
~/.claude/skills/design/protogen_stack.md
  └── Defines: FormSchema pattern, Driver/Controller, frontend stack
  
agents/PROTOGEN_STACK.md
  └── Documents: Which c1 Form fields we use, what we simplified, storage choice
  └── References: Global protogen_stack.md
  
agents/DESIGN_WEB.md
  └── Documents: Session/Cell/Workspace model, re-execution, UI layout
  └── Uses: Form pattern (doesn't re-explain it)
```

This separation means:
- Someone learning protogen stack reads the global skill
- Someone joining this project reads local notes to see our choices
- Someone understanding the feature reads domain design
