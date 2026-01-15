# Skill: Incomplete Work Discovery and Resolution

## Purpose

Systematically locate, analyze, and resolve incomplete work markers in a codebase (TODO, FIXME, TBD, XXX, HACK, stub, placeholder, etc.).

## Trigger

User asks to:
- "Find all TODOs"
- "What's incomplete in this codebase?"
- "Show me unfinished work"
- "Audit incomplete items"
- "Find and fix TODOs"

## Phase 1: Discovery

### Search Patterns

Scan for these markers (case-insensitive where sensible):

**Explicit markers:**
- `TODO` / `TODO:` / `TODO(author)`
- `FIXME` / `FIX` / `FIXME:`
- `XXX` / `XXX:`
- `HACK` / `HACK:`
- `BUG` / `BUG:`
- `TBD` / `TBD:`
- `UNDONE`
- `BROKEN`
- `REFACTOR`
- `OPTIMIZE`
- `REVIEW`

**Implicit markers:**
- `placeholder`
- `stub`
- `mock` (in non-test files)
- `dummy`
- `temporary` / `temp`
- `remove this` / `delete this`
- `not implemented`
- `unimplemented`
- `pending`

**Panic/error markers:**
- `panic("not implemented")`
- `panic("TODO")`
- `throw new Error("not implemented")`
- `raise NotImplementedError`
- `unimplemented!()`

### Search Commands

```bash
# Primary search (explicit markers) - excludes vendored/generated code
grep -rn --include="*.go" --include="*.ts" --include="*.tsx" --include="*.js" --include="*.py" --include="*.rs" --include="*.java" --include="*.c" --include="*.cpp" --include="*.h" \
  --exclude-dir=vendor --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist --exclude-dir=build \
  --exclude="*.pb.go" --exclude="*.pb.validate.go" --exclude="*_gen.go" --exclude="*.generated.*" --exclude="*connect.go" \
  -E "(TODO|FIXME|XXX|HACK|TBD|UNDONE|BROKEN|REFACTOR|OPTIMIZE|REVIEW)[\s:(\[]" .

# Secondary search (implicit markers) - excludes vendored/generated code
grep -rn --include="*.go" --include="*.ts" --include="*.tsx" --include="*.js" --include="*.py" \
  --exclude-dir=vendor --exclude-dir=node_modules --exclude-dir=.next --exclude-dir=dist --exclude-dir=build \
  --exclude="*.pb.go" --exclude="*.pb.validate.go" --exclude="*_gen.go" --exclude="*.generated.*" \
  -iE "(placeholder|stub|dummy|temporary|not.?implemented|unimplemented|pending)" .

# Panic/error markers - excludes vendored/generated code
grep -rn --include="*.go" \
  --exclude-dir=vendor --exclude-dir=node_modules \
  --exclude="*.pb.go" --exclude="*connect.go" \
  'panic\(".*TODO\|not.implemented\|unimplemented' .
```

### Directories and Files to Skip

Always exclude these from search results:

**Vendored dependencies:**
- `vendor/`
- `node_modules/`
- `third_party/`
- `.next/`
- `dist/`
- `build/`

**Auto-generated code:**
- `*.pb.go` (protobuf)
- `*.pb.validate.go` (protobuf validation)
- `*_gen.go` (code generators)
- `*.generated.*` (various generators)
- `*connect.go` (Connect-RPC generated handlers)
- `gen/` directories containing generated TypeScript/JS from protos

**Test fixtures and mocks:**
- Results in `*_test.go` files mentioning "stub" or "mock" are usually intentional
- Filter these out or mark as "expected" in the audit

### Output Format

Present findings as a structured table:

```
## Incomplete Work Audit

Found N items across M files.

| # | File:Line | Type | Context | Priority |
|---|-----------|------|---------|----------|
| 1 | pkg/api/service.go:249 | TODO | "Re-execute subsequent cells if any" | Medium |
| 2 | src/lib/client.ts:45 | FIXME | "Handle reconnection" | High |
| 3 | pkg/storage/db.go:123 | stub | Empty function body | High |
...

### By Priority
- **High (blocking)**: 3 items - stubs, panic("not implemented"), broken
- **Medium (functionality gap)**: 5 items - TODOs with clear scope  
- **Low (improvement)**: 8 items - OPTIMIZE, REFACTOR, nice-to-have

### By Category
- API/Service: 4 items
- Storage: 2 items
- Frontend: 6 items
- Tests: 4 items
```

## Phase 2: Planning (User-Triggered)

When user selects items for planning (e.g., "plan items 1, 3, 5" or "plan all high priority"):

### For Each Selected Item

1. **Read surrounding context** (50-100 lines around the marker)
2. **Identify dependencies** - what does this code touch?
3. **Assess scope** - is this a 5-line fix or a multi-file refactor?
4. **Write implementation plan** as a code comment

### Plan Comment Format

Replace the original marker with an expanded plan:

```go
// TODO(original): Re-execute subsequent cells if any
```

Becomes:

```go
// PLAN: Re-execute subsequent cells after form submission
// 
// Context: When a form is submitted in an agent cell, subsequent cells
// that depended on that form's output need to be re-executed.
//
// Implementation:
// 1. After updating cell with form response (line 246), find all cells
//    with executionSequence > targetCell.executionSequence
// 2. For each subsequent cell:
//    a. If it's a user cell, mark as superseded (user will need to re-confirm)
//    b. If it's an agent cell, delete it (will be regenerated)
// 3. Call generateAgentResponse() to produce new agent cell
// 4. Stream new events to client
//
// Files affected:
// - pkg/api/agent/service.go (this file)
// - Potentially: pkg/controller/storagev2/controller/storage.go (bulk delete)
//
// Dependencies:
// - Session.Cells must be mutable
// - Storage must support DeleteCellsAfter(sessionID, sequence)
//
// Estimated scope: ~50 lines, 1-2 files
// Priority: Medium (feature works without, but branching is incomplete)
//
// Original: TODO: Re-execute subsequent cells if any
```

### Plan Quality Checklist

Each plan MUST include:
- [ ] Clear description of what needs to be done
- [ ] Step-by-step implementation approach
- [ ] Files that will be affected
- [ ] Dependencies and prerequisites
- [ ] Scope estimate (lines, files, complexity)
- [ ] Priority assessment with rationale
- [ ] Original marker preserved for reference

## Phase 3: Implementation (User-Triggered)

When user selects items to implement (e.g., "implement item 1" or "implement all planned items"):

### Workflow

1. **Confirm scope**: "Item 1 affects 2 files, ~50 lines. Proceed?"
2. **Read all affected files** into context
3. **Implement incrementally**:
   - Make changes to first file
   - Run tests if available
   - Proceed to next file
4. **Remove the PLAN comment** after implementation
5. **Add brief implementation note** if helpful:
   ```go
   // Implemented: Form submission triggers re-execution of subsequent cells
   // See SubmitForm() for the full flow
   ```
6. **Run tests** to verify
7. **Report completion** with summary of changes

### Implementation Guidelines

- Follow existing code patterns in the file
- Maintain consistent error handling style
- Add tests if the codebase has test coverage
- Don't gold-plate - implement what's in the plan, no more
- If plan proves inadequate, update plan first, then implement

## Interaction Flow

### Example 1: Discovery Only

```
User: find all incomplete work

Claude: [runs grep searches]

## Incomplete Work Audit

Found 12 items across 6 files.

| # | File:Line | Type | Context | Priority |
|---|-----------|------|---------|----------|
| 1 | pkg/api/agent/service.go:249 | TODO | "Re-execute subsequent cells if any" | Medium |
| 2 | pkg/api/agent/service.go:312 | TODO | "Handle tool use results" | Medium |
| 3 | web/src/lib/useSession.ts:89 | FIXME | "Handle stream errors gracefully" | High |
...

### Summary
- High: 3 items (should fix before release)
- Medium: 6 items (functionality gaps)
- Low: 3 items (nice-to-have)

Would you like me to:
- Plan specific items? (e.g., "plan 1, 3")
- Plan by priority? (e.g., "plan all high")
- Implement items? (requires planning first)
```

### Example 2: Planning

```
User: plan items 1 and 3

Claude: [reads context around each item, writes PLAN comments]

Planned 2 items:

1. pkg/api/agent/service.go:249 - Added 15-line implementation plan
2. web/src/lib/useSession.ts:89 - Added 12-line implementation plan

Plans written as comments in the source files. Review them with:
  view pkg/api/agent/service.go:240-265
  view web/src/lib/useSession.ts:80-105

Ready to implement? Say "implement 1" or "implement 1, 3"
```

### Example 3: Implementation

```
User: implement 1

Claude: [reads plan, implements, runs tests]

Implemented item 1: Re-execute subsequent cells after form submission

Changes:
- pkg/api/agent/service.go: +47 lines in SubmitForm()
- pkg/controller/storagev2/driver.go: +1 method (DeleteCellsAfter)
- pkg/controller/storagev2/controller/storage.go: +15 lines

Tests: All passing

The PLAN comment has been removed. Implementation summary left in place.
```

## Edge Cases

### Large Codebases
If search returns >50 items:
- Group by directory/module first
- Ask user which areas to focus on
- Support filtering: "find TODOs in pkg/api only"

### Ambiguous Markers
Some matches may be false positives (e.g., "TODO" in a string literal, documentation about TODOs):
- Show context in the table
- Let user exclude items: "exclude 5, 7"
- Learn from exclusions for future searches

### Already-Planned Items
If an item already has a PLAN comment:
- Show as "Planned" in the status column
- Skip planning phase, go directly to implementation option

### Circular Dependencies
If implementing item A requires item B to be done first:
- Detect during planning phase
- Note dependency in plan
- Suggest implementation order

## Anti-Patterns

**Don't:**
- Auto-implement without user confirmation
- Remove TODO comments without implementing
- Add new TODOs while resolving existing ones
- Plan items that require external decisions (API design, etc.)
- Estimate time/effort (per global preferences)

**Do:**
- Present findings neutrally (user decides priority)
- Preserve original markers in plans for traceability
- Run tests after each implementation
- Stop and ask if implementation deviates significantly from plan
