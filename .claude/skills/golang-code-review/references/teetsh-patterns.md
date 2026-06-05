# Teetsh API Code Patterns

This document captures the specific coding patterns and preferences used in the Teetsh API codebase.

## Function Design

Extract complex logic into properly named subfunctions that maintain the same level of abstraction:
- Each function should have a single, clear responsibility
- Avoid mixing different levels of abstraction within the same function
- Use descriptive function names that clearly indicate purpose

Example pattern:
```go
// Good: Clear separation of concerns with consistent abstraction levels
func GetPromotionCodeInfo(code string) (*PromotionBonusResult, error) {
    if trialBonus := checkTrialBonus(); trialBonus != nil {
        return trialBonus, nil
    }

    codesToCheck := getCodesToCheck(code)

    if result := checkAmbassadorCode(codesToCheck); result != nil {
        return result, nil
    }

    if result := checkParrainageCode(codesToCheck); result != nil {
        return result, nil
    }

    return checkStripePromotionCode(code)
}
```

## Return Value Patterns

Prefer returning a struct instead of multiple return values when returning related data:
- Create a dedicated result struct with clear field names
- Include convenience methods on the struct when appropriate
- Improves type safety, readability, and maintainability

```go
// Good: Struct with related fields and convenience method
type PromotionBonusResult struct {
    PromotionType PromotionType
    CodeUsed      string
    BonusDays     int
    BonusMonths   int
}

func (p PromotionBonusResult) HasPromotion() bool {
    return p.PromotionType != ""
}

// Bad: Multiple return values
func GetPromotion(code string) (PromotionType, string, int, int, error) {
    // ...
}
```

## Testing Patterns

### Use Vanilla Go Testing
- Use standard library testing functions (`t.Error()`, `t.Fatal()`, `t.Errorf()`)
- Avoid third-party assertion libraries like `testify/assert`
- More explicit control flow and better stack traces

```go
// Good: Vanilla Go testing
if result != expected {
    t.Errorf("Expected %v, got %v", expected, result)
}

if subscription == nil {
    t.Fatal("subscription should exist")
}

// Bad: Assertion libraries
assert.Equal(t, expected, result)
assert.NotNil(t, subscription)
```

### Behavior-Driven Testing
- Verify outcomes rather than implementation details
- Prefer stateful fake implementations over mocks that just track method calls
- Test the final state of the system, not the intermediate steps

```go
// Good: Testing outcomes
subscription := fakeStripe.GetSubscription("sub_123")
if subscription == nil {
    t.Fatal("subscription should exist")
}
if subscription.CustomerID != "cus_123" {
    t.Errorf("Expected CustomerID cus_123, got %s", subscription.CustomerID)
}

// Bad: Testing implementation details
mock.AssertCalled(t, "CreateSubscription", "cus_123")
```

### Test Organization
- Use behavior-driven test names that describe what is being tested
- Group related test cases using subtests with `t.Run()`
- Setup/teardown should be minimal and focused

## Comment Guidelines

Write comments that explain WHY, not WHAT:

### Keep Necessary Comments:
- Function documentation for exported functions (Go conventions)
- Type/struct documentation explaining purpose and usage
- Complex business logic explanations where intent isn't obvious
- API contracts, invariants, and important assumptions
- TODOs with context about what needs to be done and why
- Warnings about non-obvious behavior or gotchas

### Avoid Unnecessary Comments:
- Comments that repeat what code obviously does
- Step-by-step comments for simple operations
- Redundant inline comments in straightforward functions
- Type conversion comments obvious from function names

```go
// Good: Explains why
// We need to normalize field names to handle French accents and case variations
// so "Modalité", "modalite", and "MODALITÉ" are recognized as the same field
func normalizeFieldName(name string) string {
    lower := strings.ToLower(name)
    noAccents := removeAccents(lower)
    return strings.TrimSpace(noAccents)
}

// Bad: Describes what (obvious from code)
func normalizeFieldName(name string) string {
    // Convert to lowercase
    lower := strings.ToLower(name)
    // Remove accents
    noAccents := removeAccents(lower)
    // Trim whitespace
    trimmed := strings.TrimSpace(noAccents)
    // Return the result
    return trimmed
}
```

### Struct Field Comments
Use inline comments sparingly, only when field name doesn't clearly convey purpose:

```go
type CustomFieldMappingPlan struct {
    ExistingMappings map[uuid.UUID]uuid.UUID        // Maps original field IDs to existing field IDs
    FieldsToCreate   map[uuid.UUID]CustomFieldInfo  // Fields that need to be created
    NextPosition     int                            // Starting position for new fields
}
```

## Architecture Patterns

### REST API Structure
Each domain follows this pattern:
- `pkg/{domain}/` - Core business logic, entities, and repository interfaces
- `pkg/{domain}/repo.go` - Database operations
- `pkg/{domain}/service.go` - Business logic
- `pkg/rest/{domain}/` - HTTP handlers and endpoints
- `pkg/rest/{domain}/viewmodel/` - Request/response DTOs

### Multi-tenancy
- Schools are isolated by `school_id`
- Most queries should include school context
- Ensure proper authorization checks

### Analytics & Event Tracking
All tracked events MUST be defined as constants in `pkg/externals/tracker/client.go`:
- Event naming convention: `Entity:action` or `Entity:subEntity:action`
- Examples: `EdtTemplate:used`, `Programmation:exportPdf`, `CahierJournal:AIAssist:prompt`
- When tracking template usage, always include `template_document_id`, `template_name`, `source`
- Track events asynchronously with `go h.tracker.Track(...)`
