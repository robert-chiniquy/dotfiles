# Design Skill: Systematic Feature Design

## Purpose

Apply a rigorous 10-step methodology for designing new features and systems, ensuring fundamentals are addressed before polish.

## When to Apply

- Starting new feature design
- Planning major refactoring
- Designing developer-facing tools or APIs
- Any work requiring architectural decisions

---

## The 11-Step Process

### 1. Research
Survey prior art and understand the landscape.

**Activities:**
- Survey comparable platforms/tools
- Create comparison matrices
- Document sources and approaches

**Output:** Research document with comparison tables.

### 2. Compare
Find patterns and identify gaps across research.

**Activities:**
- Extract common themes
- Note where platforms differ and why
- Identify gaps vs. prior art
- Consider different personas

**Output:** Patterns document, differences document, gap analysis.

### 3. Refine (Qualitative → Quantitative)
Turn subjective terms into measurable attributes.

**Activities:**
- Identify qualitative terms ("lovable", "seamless", "gold standard")
- For each term, enumerate specific observable attributes
- Each attribute must be testable or measurable

**Example:**
```
Before: "The CLI should be lovable"

After:  "Lovable" means:
        - Completes common tasks in ≤3 commands
        - Provides clear error messages with suggested fixes
        - Shows progress for operations >2 seconds
        - Supports both interactive and scriptable modes
```

### 3.5. Design Documentation Ontology
Create information architecture before writing content.

**Activities:**
- Map all knowledge domains in the system
- Organize into logical hierarchy (Getting Started → Advanced)
- Identify relationships between sections
- Plan for both MVP and future expansion
- Ensure ontology covers all personas (developers, users, operators)

**Output:** Documentation ontology (nested bullet structure).

**Why this matters:**
- Reveals missing knowledge areas early
- Ensures coherent information architecture
- Guides content creation priorities
- Prevents documentation silos and duplication

**Example structure:**
```
1. Getting Started
   - Prerequisites
   - Quickstart
2. Core Concepts
   - Architecture
   - Key terminology
3. Building
   - Implementation guide
   - API reference
4. Deployment
   - Patterns
   - Production readiness
```

### 4. Ideate (Feature Wishlist)
Generate concrete features that satisfy refined attributes.

**Activities:**
- For each attribute, brainstorm features
- Write as concrete, implementable items
- Include example CLI syntax, UI mockups, or code snippets
- Map features back to attributes

**Output:** Feature wishlist with examples and traceability.

### 5. Discover (Explore Existing Codebase)
Understand what already exists before designing new things.

**Activities:**
- Explore relevant codebases
- Document current architecture
- Identify reusable components
- Find existing patterns and conventions

**Output:** Architecture summary, reusable component list, conventions.

**This step often triggers doubling back** — discovering existing infrastructure changes the design.

### 6. Implement (Detailed Paths)
Create detailed implementation plans for each feature.

**Activities:**
- For each feature, list possible implementation approaches
- For each approach, document:
  - End-state architecture
  - Required code changes (file by file)
  - Tradeoffs
- Recommend an approach

**Output:** Implementation document with multiple paths per feature.

### 7. Critique
Find problems before building. (See separate critique skill.)

**Activities:**
- Apply three lenses: Complexity, Fundamentals, Feasibility
- Create explicit cut list
- Define minimal v1 scope

**Expected outcome:** 30-50% feature reduction, clearer design.

### 8. Double Back
Return to earlier steps with new knowledge.

**Triggers:**

| Trigger | Go Back To |
|---------|------------|
| Critique reveals missing fundamentals | Step 3 (Refine) or Step 4 (Ideate) |
| Discovery finds existing infrastructure | Step 4 (Ideate) |
| Stakeholder answers change assumptions | Step 4 (Ideate) |
| New persona or use case identified | Step 2 (Compare) |
| Scope too large | Step 4 (Ideate) with constraints |

**This is where most value is created.** First pass is always naive.

### 9. Consolidate
Merge all learnings into coherent final plan.

**Activities:**
- Create new implementation document incorporating:
  - Original good ideas
  - Critique findings (cuts, simplifications)
  - Double-back decisions
  - Architecture decisions
- Mark superseded documents
- Ensure vertical slices (ship value incrementally)

**Output:** Consolidated implementation plan (v2).

### 10. Document Decisions
Capture decisions for future reference.

**Activities:**
- Record each architectural decision with:
  - Context (what prompted the decision)
  - Options considered
  - Decision made
  - Rationale
- Update index/README
- Ensure cross-references between documents

**Output:** Architecture decision records, updated index.

### 11. Design Documentation Ontology
Create complete information architecture.

**Activities:**
- Map all knowledge domains covered by the system
- Organize into logical hierarchy (beginner → advanced)
- Identify cross-references and relationships
- Plan for both MVP and future content expansion
- Ensure coverage for all personas

**Output:** Documentation ontology as nested structure.

**Why this is a separate step:**
- Often reveals missing functional requirements ("we need to document X, but X doesn't exist yet")
- Ensures documentation is comprehensive before writing content
- Guides prioritization (what docs are needed for MVP vs later)
- Creates shared vocabulary for team

**This step may trigger doubling back:** If ontology reveals missing features or personas, return to earlier steps.

---

## The Phase Framework

Phases represent complete passes through the design process. When you revisit a step (e.g., after critique triggers a double-back), that work belongs to the next phase.

### Phase Numbering

```
PHASE_1_RESEARCH.md      # Initial research
PHASE_1_COMPARE.md       # Initial comparison
PHASE_1_IDEATE.md        # Initial feature wishlist
PHASE_1_CRITIQUE.md      # First critique pass
  ↓ (critique reveals gaps, triggers double-back)
PHASE_2_IDEATE.md        # Revised wishlist addressing critique
PHASE_2_IMPLEMENT.md     # Implementation plan v2
PHASE_2_CRITIQUE.md      # Second critique pass
  ↓ (architect review, more refinements)
PHASE_3_IMPLEMENT.md     # Final implementation plan
```

### Rules

1. **Never revise prior phase documents** - Create new document in next phase
2. **Reference prior phases** - New docs should cite what they supersede
3. **Phase number increments on any double-back** - Not just major rewrites
4. **Final phase gets `_FINAL` suffix** - Indicates approved plan

### Naming Convention

```
PHASE_<N>_<STEP>.md           # Standard step document
PHASE_<N>_<STEP>_REVISED.md   # Minor revision within same phase
PHASE_<N>_<STEP>_FINAL.md     # Approved/frozen document
```

### Example Directory Structure

```
docs/design/
├── PHASE_1_RESEARCH.md
├── PHASE_1_COMPARE.md
├── PHASE_1_IDEATE.md
├── PHASE_1_CRITIQUE.md           # Identifies issues
├── PHASE_2_IDEATE.md             # Addresses critique
├── PHASE_2_IMPLEMENT.md
├── PHASE_2_CRITIQUE.md           # Architect review
├── PHASE_2_CRITIQUE_REVISED.md   # Addresses review findings
├── PHASE_2_CONSOLIDATE.md
├── PHASE_2_FINAL.md              # Approved implementation plan
└── README.md                     # Index with phase progression
```

### When to Increment Phase

| Trigger | Action |
|---------|--------|
| Critique identifies issues requiring ideation changes | New phase |
| Architect review with structural changes | New phase |
| Stakeholder feedback changing requirements | New phase |
| Discovery invalidates assumptions | New phase |
| Minor clarifications or typo fixes | `_REVISED` suffix, same phase |

### Version Suffixes (Alternative Pattern)

For documents that evolve without the full design process (e.g., backlogs, status reports), use version suffixes:

```
BACKLOG.md      # Initial (implied V1)
BACKLOG_V2.md   # After significant changes
BACKLOG_V9.md   # Many iterations later
```

---

## The Level Framework

Use this to ensure fundamentals are addressed before polish:

```
┌─────────────────────────────────────────────┐
│  LEVEL 2: Polish                            │
│  - Progress indicators                      │
│  - Error messages                           │
│  - Shell completion                         │
│  - Output formatting                        │
├─────────────────────────────────────────────┤
│  LEVEL 1: Workflow                          │
│  - Local development                        │
│  - Testing                                  │
│  - Deployment commands                      │
│  - Debugging                                │
├─────────────────────────────────────────────┤
│  LEVEL 0: Platform Architecture             │
│  - Authentication / Authorization           │
│  - Data model / Schema                      │
│  - Core APIs                                │
│  - Deployment lifecycle                     │
│  - Security model                           │
└─────────────────────────────────────────────┘
```

**Rule:** Never plan Level 2 before Level 0 is addressed.

---

## Signs You Need to Double Back

| Symptom | Likely Cause | Action |
|---------|--------------|--------|
| "How does auth work?" during implementation | Level 0 gap | Back to Step 3-4 |
| Discovering existing code that does what you planned | Inadequate discovery | Back to Step 5 |
| Estimate exceeds budget significantly | Over-scoped | Back to Step 7 (critique) |
| Multiple teams confused about approach | Poor documentation | Back to Step 10 |
| Feature has no user asking for it | Scope creep | Back to Step 7 (cut list) |
| Creating packages for <100 lines of code | Over-engineering | Back to Step 7 |

---

## Key Principles

1. **Steps 7-8 create the most value** - Critique and doubling back transform naive plans into good plans
2. **Level 0 before Level 2** - Platform architecture before polish
3. **Concrete over abstract** - Every feature needs examples
4. **Additive documentation** - Never delete research, create superseding documents
5. **Vertical slices** - Ship value incrementally, not horizontal layers

---

## Anti-Patterns

- Skipping research ("we'll figure it out")
- Leaving qualitative terms vague ("we'll know it when we see it")
- Features without examples ("add hot reload" — how exactly?)
- Designing without reading existing code
- Only one implementation path (no alternatives considered)
- Skipping critique ("we're confident in the plan")
- Refusing to revisit decisions ("we already decided that")
- Keeping multiple conflicting documents

---

## Self-Check

Before claiming design is complete, verify:

- [ ] Comparison matrix created with 3+ platforms
- [ ] Qualitative terms defined with measurable attributes
- [ ] Documentation ontology designed (all knowledge domains mapped)
- [ ] Every feature has concrete example
- [ ] Existing codebase explored for reusable components
- [ ] Level 0 questions answered (auth, schema, deployment)
- [ ] Critique performed with 30-50% feature cuts
- [ ] At least one double-back iteration completed
- [ ] Superseded documents marked clearly
- [ ] Architectural decisions documented with rationale
- [ ] Documentation ontology covers all personas and use cases
