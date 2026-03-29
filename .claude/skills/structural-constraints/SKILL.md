---
name: structural-constraints
description: |
  Guide architecture toward compile-time safety over runtime checks. Use when
  designing subsystems, reviewing architecture, or choosing between runtime
  and compile-time enforcement. Covers type systems, enums, interfaces.
---

# Structural Constraints

Push decisions to compile time. Every runtime check is a bug waiting for the case you forgot.

## Checklist

When designing or reviewing:

- [ ] Can this `if` be an enum/sum type?
- [ ] Can this string be a typed constant?
- [ ] Can this map lookup be a struct field?
- [ ] Can this nil check be eliminated by construction?
- [ ] Does this interface have exactly one implementation? (Smell.)
- [ ] Can the type system prevent invalid states?

## Patterns

* **Enum over string.** `Status` enum, not `status string`. The compiler catches typos.
* **Constructor validation.** Parse, don't validate. `NewEmail(s)` returns `(Email, error)`, not `string`.
* **Required fields via types.** `Config{Required: X}` not `Config{Optional: &X}`.
* **State machines via types.** `Draft -> Published -> Archived`, not `status = "published"`.

## Common Mistakes

1. Wrapping everything in interfaces "for testing" — if there's one impl, use the concrete type
2. Using `map[string]interface{}` when a struct would work — you lose all compile-time checking
3. Adding runtime validation for invariants the type system could enforce
4. Creating "stringly typed" APIs where enums would prevent invalid states
5. Using `interface{}` / `any` to avoid thinking about the actual type
6. Feature flags as strings instead of typed constants
7. Optional fields that are always present in practice — make them required
8. Using error returns for conditions that should be structurally impossible
