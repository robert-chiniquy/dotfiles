# Multi-Subproject Management

Patterns for complex repos with multiple interrelated subprojects.

## Subproject Isolation

Each subproject should be independently usable.

Requirements:
- Own go.mod (or equivalent)
- Own tests (runnable via root Makefile)
- Own CLAUDE.md if it has code
- Own DATA_SOURCES.md if it consulted external sources
- Clear interface to other subprojects

Dependencies between subprojects: explicit replace directives during dev, proper module paths for release.

## Separation of Concerns

Better to create new subproject than pollute existing one.

When adding functionality spanning multiple concerns:
1. Does it belong in existing subproject's responsibility?
2. If NO: Create new subproject for integration layer
3. If YES: Add, but watch for growing scope

Signs subproject needs splitting:
- Import cycles between packages
- Tests require mocking unrelated concerns
- Multiple independent main entry points
- Different deployment patterns for parts

Integration layers belong in own subprojects:
- `classifiers-egraph-bridge/` not `classifiers/egraph.go`
- `datalog-smt-bridge/` not `datalog/smt.go`
- `integration/` for unified services

## Existing Code Discovery

Search existing codebase before assuming missing.

Before implementing:
1. Search for related naming conventions (`*_test.go`, `laws_*.go`)
2. Check alternative patterns
3. Grep for function names

```bash
glob **/*test*.go
grep -r "func.*Property" --include="*.go"
```

Example mistake: Assuming no property tests existed because they used `laws_*.go` not `*_test.go`.

## Unifying Algorithms

When implementing related operations, find the unifying algorithm.

Example: All set operations use same product construction with different painters:

| Operation | Painter |
|-----------|---------|
| Union | either accepts |
| Intersection | both accept |
| Difference | left accepts, right doesn't |
| SymmetricDifference | exactly one accepts |

One algorithm, many operations. Reduces bugs and maintenance.

## Avoiding Mutual Recursion

Ensure call graph is acyclic for derived predicates.

Dangerous:
```go
func (a *TypeA) Equals(b *TypeA) bool {
    return a.SymmetricDifference(b).IsEmpty()
}

func (b *TypeB) IsEmpty() bool {
    return b.left.Equals(b.right)  // Infinite loop!
}
```

Solutions:
- Compute via different mechanism (DFA product)
- Use conservative approximations
- Separate structural ops from derived predicates

## Conservative Approximation

When exact computation is expensive/impossible, return conservative estimates.

Examples:
- `Relation()` returning "might overlap" instead of precise relation
- `IsEmpty()` returning false when unable to determine
- `IsSubsetOf()` returning false when unable to verify

Mark clearly in docs. Conservative = safe (won't cause incorrect behavior) but may miss optimizations.

## Cross-Validation Testing

Different representations verify each other.

When you have multiple implementations (DFA, Datalog, SMT):
1. Generate random test cases
2. Evaluate with each representation
3. Assert results agree
4. On disagreement, minimize to find bug

Catches bugs single-representation tests miss.
