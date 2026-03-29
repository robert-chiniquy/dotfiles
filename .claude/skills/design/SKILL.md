---
name: design
disable-model-invocation: true
description: |
  Feature design methodology. Covers research, comparison, requirements
  refinement, ideation, implementation planning, and DX completeness.
  Use when starting a new feature, planning a refactor, or designing APIs.
  Invoked via /design [topic].
argument-hint: "[feature or topic]"
---

# Design

Structured approach to feature design. Not every step applies to every feature.
Skip steps that don't add value. The sequence matters more than completeness.

## Steps

### 1. Research
What exists? Read the codebase. Read competing implementations. Read the docs.
Capture findings in DATA_SOURCES.md.

### 2. Compare
How do others solve this? Build a comparison matrix: feature vs platform.
Look for patterns across 3+ implementations.

### 3. Refine
Convert vague requirements ("make it fast") into measurable attributes
("p99 latency under 200ms"). Challenge ambiguous terms.

### 4. Ideate
Generate concrete features with examples. Each feature gets: name, one-line
description, CLI example or API shape, which attribute it serves.

### 5. Scope
Cut aggressively. What's the smallest thing that delivers the core value?
Use the Level Framework: Level 0 (platform), Level 1 (workflow), Level 2 (polish).
Build Level 0 first.

### 6. Plan
Vertical slices, not horizontal layers. Each slice is deployable and testable.
Count lines of code per slice. If a slice exceeds 500 lines, split it.

### 7. Critique
Run /critique on the plan before implementing.

### 8. DX Check
Every developer-facing feature needs three things:
* **Tools** — CLI commands, SDK methods, APIs
* **Documentation** — how to use it, with examples
* **Agent support** — can an LLM agent use it effectively?

If any leg is missing, the feature is incomplete.

## Signs You Need to Double Back

* The plan has more than 10 slices
* You're building Level 2 before Level 0 exists
* The critique found more than 5 problems
* You can't demo the core value in under 2 minutes
* The scope grew since you started
