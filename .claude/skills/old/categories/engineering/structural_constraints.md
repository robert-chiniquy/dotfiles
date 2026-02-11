# Engineering Skill: Structural Constraints

## Philosophy

The best engineering doesn't rely on developer discipline to prevent mistakes—it makes mistakes **structurally impossible**. Instead of "remember to check X", the system physically cannot compile or execute without X.

This skill guides architectural decisions toward compile-time safety, type-system enforcement, and structural guarantees over runtime checks.

---

## Core Principle: Defense Through Structure

| Weak (Discipline-Based) | Strong (Structure-Based) |
|-------------------------|--------------------------|
| "Remember to add tenant_id" | Query builder auto-injects tenant_id from scoped context |
| "Always validate input" | Proto annotations generate validation at compile time |
| "Don't forget to close connections" | Resource scoping via defer or RAII patterns |
| "Check permissions before access" | Type system requires permission token to call function |

---

## Patterns

### 1. Scoped Context Propagation

Pass identity/context through the call stack via explicit scoping, not globals.

```go
// Weak: Global or implicit context
func GetUser(id string) (*User, error) {
    tenantID := getCurrentTenant()  // Where does this come from?
    // ...
}

// Strong: Explicit scoped context
func (c *ScopedController) GetUser(ctx context.Context, id string) (*User, error) {
    // c.passport was set via WithPassport(p) - can't be nil
    // All DB queries automatically scoped to c.passport.TenantId
}
```

### 2. Dangerous Operations Named Explicitly

When escape hatches are necessary, make them loud.

```go
// Safe path (normal)
conn := db.WithPassport(p)

// Escape hatch (requires justification)
conn := db.WithDangerousCrossTenant(p)  // Panics if not system tenant
```

The "Dangerous" prefix triggers code review attention.

### 3. Single Source of Truth via Code Generation

Define schemas once, generate everything else.

- Database schemas generated from type definitions
- API validation generated from schema annotations
- Client code generated from API definitions

Drift becomes impossible because there's only one source.

### 4. Fail-Closed by Default

Security-critical paths return errors immediately, never continue silently.

```go
// Weak: Default allow on error
if err != nil {
    log.Warn(err)
    return true  // Dangerous: allows on failure
}

// Strong: Default deny on error
if err != nil {
    return false, fmt.Errorf("validation failed: %w", err)
}
```

### 5. Interface Segregation for Minimal Authority

Depend on the smallest interface that satisfies requirements.

```go
// Weak: Accept full controller when you only read
func Process(ctrl FullController) { ... }

// Strong: Accept minimal interface
func Process(reader Reader) { ... }
```

Smaller interfaces are easier to mock, audit, and constrain.

---

## Application Checklist

When designing systems, ask:

1. **Can this mistake be caught at compile time?** (types, interfaces, generics)
2. **Can this constraint be enforced by the query/ORM layer?** (auto-injection)
3. **Is the unsafe path obviously named?** (Dangerous*, Unsafe*, Raw*)
4. **Does the error path deny by default?** (fail-closed)
5. **Is there a single source of truth?** (generation over synchronization)

---

## Anti-Patterns to Avoid

- **Runtime checks for compile-time guarantees** - If the type system can enforce it, use types
- **Implicit context** - Globals, thread-locals, ambient authority
- **Soft failures** - Logging errors and continuing
- **Ceremony without enforcement** - Documentation that says "must" without compiler backing
- **Multiple sources of truth** - Manual sync between schema, code, and docs

---

## When to Apply

This skill applies when:

- Designing new subsystems or abstractions
- Reviewing architectural decisions
- Evaluating tradeoffs between flexibility and safety
- Choosing between runtime and compile-time checks

It does not apply to quick scripts, prototypes, or exploratory code where the overhead isn't justified.
