---
name: structural-constraints
description: |
  Guide architectural decisions toward compile-time safety and structural
  guarantees over runtime checks. Use when designing new subsystems,
  reviewing architectural decisions, or choosing between runtime and
  compile-time enforcement. Not for quick scripts or prototypes.
---

# Structural Constraints

The best engineering makes mistakes structurally impossible rather than relying on developer discipline.

Patterns: Scoped context propagation (explicit over globals), dangerous operations named explicitly (WithDangerousCrossTenant), single source of truth via code generation, fail-closed by default, interface segregation for minimal authority.

Checklist: Can this mistake be caught at compile time? Can this constraint be enforced by the query/ORM layer? Is the unsafe path obviously named? Does the error path deny by default? Is there a single source of truth?
