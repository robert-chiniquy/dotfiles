# Documentation Process

8-step process for writing documentation.

```
0. PRE-FLIGHT    Check prerequisites, map dependencies
1. SCOPE         Define topic, identify persona
2. SOURCE        Gather verified sources, mark confidence
3. OUTLINE       Structure using 7-step pattern
4. DRAFT         Write first pass with source traceability
5. VERIFY        Confirm claims against current source
6. CONTRADICTION Note inconsistencies, explain why
7. CONNECT       Add cross-references, navigation
8. PUBLISH       Remove internal references
```

## Step 0: Pre-Flight

Checklist:
- [ ] Prerequisite docs exist
- [ ] This doc's place in reader journeys is clear
- [ ] Glossary terms defined
- [ ] No blocking TODOs

```markdown
## Dependencies
Requires: [docs that must exist first]
Unlocks: [docs that can be written after]
```

## Step 1: Scope

Questions:
- What specific topic?
- Which persona?
- What can reader do after reading?
- What prerequisite knowledge assumed?

```markdown
## Topic: [Name]
Persona: [Developer, Operator, Both]
Reader Goal: [e.g., "Deploy connector in service mode"]
Prerequisites: [e.g., "Completed Getting Started"]
```

## Step 2: Source

Authority hierarchy:
| Source | Level | Use For |
|--------|-------|---------|
| Source code | Ground truth | Implementation |
| Official docs | High | Concepts |
| Team knowledge | Medium | Intent |
| External docs | Medium | May be stale |

Confidence markers:
- `[VERIFIED]` - Read this session, confirmed
- `[INFERRED]` - Synthesized from multiple sources
- `[GENERATED]` - Example content
- `[UNVERIFIED]` - Believed true, not confirmed

## Step 3: Outline

Use 7-step pattern (see `doc-content.md`).

## Step 4: Draft

See `doc-content.md` for writing guidelines.

## Step 5: Verify

Checklist:
- [ ] Every code example runs
- [ ] All CLI flags exist
- [ ] All SDK methods exist
- [ ] All file paths correct
- [ ] All line citations accurate

Output: Verified draft, no `[UNVERIFIED]` markers.

## Step 6: Contradiction

Present current state in best possible light with no inaccuracy.

Bad: "All connectors support provisioning."
Good: "Connectors may support provisioning. Check capability manifest."

## Step 7: Connect

Connection types:
- Prerequisites - what to do first
- Related topics - deepen understanding
- Next steps - where to go after
- Cross-references - inline links

Navigation audit:
- [ ] Every concept links to definition
- [ ] Clear "what's next" at end
- [ ] No dead-end pages
- [ ] Prerequisites linked, not just listed

## Step 8: Publish

Checklist:
- [ ] Remove internal location references
- [ ] Replace internal citations with public links
- [ ] Remove internal statistics
- [ ] No internal project names
- [ ] All links publicly accessible

## Signals to Double Back

| Signal | Action |
|--------|--------|
| Source conflict | Stop, resolve, may need new phase |
| Missing prerequisite | Write prerequisite first |
| Scope creep | Split into multiple docs |
| Contradiction discovered | Note it, may revise earlier sections |
| Reader journey broken | Fix navigation first |
| Glossary gap | Add to glossary immediately |

When to new phase vs fix in place:
- Minor fixes: Fix in current phase
- Structural changes: New phase
- New info changing assumptions: New phase
