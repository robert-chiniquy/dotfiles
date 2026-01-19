# Documentation Merging

When two bodies of documentation cover the same domain, merge them into a unified structure that preserves content from both while eliminating redundancy.

---

## When This Applies

- Combining internal and external documentation
- Merging research documents into published docs
- Consolidating overlapping guides from different teams
- Integrating acquired product documentation

---

## Core Principle

**Find the ontology that serves readers, not the one that preserves existing structure.**

Both source documents had reasons for their structure. Neither structure is sacred. The merged result may look like neither original.

---

## Alternative: Facet by Audience

Sometimes documents serve different audiences and shouldn't be merged into one linear flow. Faceting by audience is valid when:

- Sources address distinct personas (developers vs operators vs end-users)
- Reader contexts differ enough that unified content would frustrate both groups
- Each audience needs different depth, terminology, or examples

In this case, keep separate tracks and add a **landing page** that helps readers self-select:

```
Landing Page: "Connector Documentation"
  |
  +-- "I want to build a connector" -> Developer Guide
  +-- "I want to deploy a connector" -> Operator Guide
  +-- "I want to request access" -> End-User Guide
```

The landing page replaces the need to merge. Content stays separate but discoverable.

---

## Process

### 1. Inventory Both Sources

Create a flat list of every discrete piece of content from both sources:

| Source | Section | Content Summary | Unique? |
|--------|---------|-----------------|---------|
| Doc A | Setup | Installing dependencies | No - Doc B has similar |
| Doc A | Setup | Environment variables | Yes |
| Doc B | Getting Started | Quick install | No - Doc A has similar |
| Doc B | Getting Started | First sync | Yes |

Mark each item:
- **Unique**: Only one source covers this
- **Overlapping**: Both sources cover this (pick the better version)
- **Contradictory**: Sources disagree (resolve before proceeding)

### 2. Identify the Reader's Journey

What sequence does a reader actually need? Common patterns:

- **Task-based**: Organized by what readers want to accomplish
- **Conceptual-first**: Explain the model, then show usage
- **Reference**: Alphabetical or categorical lookup

The original documents may have used different patterns. Choose one for the merged result.

### 3. Build the New Ontology

Group inventory items by reader need, not by source document:

```
Before (two separate structures):
  Doc A: Concepts -> Installation -> Usage -> Reference
  Doc B: Quick Start -> Deep Dives -> API

After (unified by reader journey):
  Getting Started (from both Quick Start sections)
  Core Concepts (from Doc A Concepts + Doc B Deep Dives)
  Building (from Doc A Usage + Doc B Deep Dives)
  Reference (from both Reference/API sections)
```

### 4. Handle Overlapping Content

When both sources cover the same topic:

1. **Compare quality**: Which is clearer, more accurate, more complete?
2. **Check for unique details**: One version may have examples the other lacks
3. **Merge the best parts**: Take the better structure, add missing details from the other
4. **Delete the inferior version entirely**: Don't leave fragments

### 5. Resolve Contradictions

When sources disagree:

1. **Identify ground truth**: What does the implementation actually do?
2. **Document the resolution**: Note why one version was chosen
3. **Update both if needed**: The contradiction may reveal a documentation gap

### 6. Preserve Provenance (During Merge Only)

While merging, track where content came from:

```markdown
<!-- Source: Doc A, Section 3.2 -->
Content here...

<!-- Source: Doc B, API Reference -->
More content...
```

Remove these markers before publication. They exist only to enable review.

---

## What to Avoid

**Interleaving at the paragraph level.** If you're alternating sentences from different sources, the merge has gone wrong. Each section should be coherent.

**Preserving both structures as "Part 1" and "Part 2".** This is concatenation, not merging. Readers shouldn't know there were two sources.

**Keeping inferior content "just in case".** If one version is clearly better, delete the other. Version control preserves history.

**Changing terminology mid-document.** If Doc A calls it "sync" and Doc B calls it "fetch", pick one and use it everywhere.

---

## Quality Checks

Before considering the merge complete:

| Check | Method |
|-------|--------|
| No orphaned content | Every inventory item appears in merged result |
| No duplicates | Search for similar headings, repeated explanations |
| Consistent terminology | Grep for synonym pairs |
| Coherent flow | Read straight through - does it make sense? |
| No source artifacts | Remove provenance markers, internal references |

---

## Kissane's Content Principles Applied

From "The Elements of Content Strategy":

| Principle | Application to Merging |
|-----------|----------------------|
| **Appropriate** | Merged content fits reader's actual context |
| **Useful** | Every section helps readers accomplish something |
| **Clear** | Merging doesn't introduce ambiguity |
| **Consistent** | Terminology, tone, and structure are uniform |
| **Concise** | Merging reduces redundancy, not increases it |

The merge succeeds when readers can't tell it was ever two documents.
