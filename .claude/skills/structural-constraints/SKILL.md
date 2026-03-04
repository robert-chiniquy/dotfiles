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

## Common Mistakes

1. **Validating at the wrong layer** — validation in the handler that should be in the type system. If a value can't be negative, use `uint`, don't check `if x < 0`.
2. **Stringly-typed APIs** — passing `string` where an enum or distinct type would make invalid states unrepresentable. `status string` vs `Status enum`.
3. **Runtime permission checks that could be compile-time** — if an operation is only valid for admins, make admin a separate type with a method, not a runtime `if user.IsAdmin()` gate.
4. **Optional fields that aren't optional** — a field required by every caller but typed as optional because one constructor doesn't set it. Fix the constructor.
5. **Global state disguised as context** — stuffing values into `context.Context` and hoping callers extract them. Use explicit function parameters.
6. **Fail-open error handling** — `if err != nil { log.Warn(err) }` then proceeding. Default must be deny/fail, not continue.
7. **Defensive copies instead of immutable types** — copying a struct to prevent mutation when the real fix is making the struct's fields unexported.
8. **Generating code then editing it** — the generated code is the source of truth. If you need to change it, change the generator input.
