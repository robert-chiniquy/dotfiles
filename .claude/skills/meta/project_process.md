# Meta Skill: Project Process

## Purpose

Define mandatory practices for all projects regardless of type or methodology. These are non-negotiable process requirements that apply universally.

## When to Apply

- Every project, always
- These requirements are cumulative with other skills

---

## Mandatory Project Artifacts

### 1. DATA_SOURCES.md

Every project MUST maintain a `DATA_SOURCES.md` file in the project root.

**Purpose:** Track provenance of all information used in the project.

**Contents - document every source from which information was acquired:**

1. **Filesystem paths** - Local files, directories, or repositories consulted
2. **URLs** - Web pages, API documentation, GitHub issues, etc.
3. **Other sources** - Conversations, meeting notes, Slack threads (with date/context)

**Format:**

```markdown
# Data Sources

## Filesystem

- `/path/to/repo/` - Description of what was learned
- `/path/to/file.go:123-456` - Specific code section referenced

## URLs

- https://example.com/docs - Description of content
- https://github.com/org/repo/issues/123 - Issue context

## Other

- [2025-01-14] Slack conversation with context - Key insight gained
```

**Rules:**

- Add sources as they are consulted, not retroactively
- Include enough context to relocate the information later
- For code, include line numbers when referencing specific sections
- For URLs, note if content may change (wiki pages, live docs)
- Date-stamp non-permanent sources

**Specificity requirements for code analysis:**

The phrase "from code analysis" is unacceptable - too vague to enable verification.

At minimum, code sources must include:
- Repository path (full path or path from home directory)
- Date of analysis
- File path within repo (for specific references)
- Line numbers (when citing specific code)

**Good:**
```markdown
| baton-okta | `/Users/rch/repo/c1in-pathway/baton-repos/baton-okta/pkg/connector/app.go:334-336` | Uses app.Id with RawId |
```

**Bad:**
```markdown
| baton-okta | from code analysis | Uses app.Id |
```

**Why:** Enables verification, supports future researchers, creates audit trail.

---

### 2. LEARNINGS.md

Every project MUST maintain a `LEARNINGS.md` file in the project root.

**Purpose:** Preserve discoveries and insights that emerge during work.

**Format:**

```markdown
# LEARNINGS.md

## 2025-01-14: Topic Name

What was learned, why it matters, how it was discovered.

Include:
- Concrete examples
- File paths with line numbers
- Code snippets
- Diagrams where helpful

## 2025-01-13: Earlier Topic

...
```

**Rules:**

- Add learnings immediately when discovered, not at end of session
- Use dated headers (## YYYY-MM-DD: Topic) for organization
- Include enough detail to reconstruct the reasoning
- Reference specific files and line numbers
- This is append-only - never delete prior learnings

**Why:** Creates knowledge base that persists beyond individual conversations. Prevents re-discovery of same insights.

---

### 3. HUMAN_ACTIONS_NEEDED.md (when applicable)

When work requires human action that blocks progress, maintain a `HUMAN_ACTIONS_NEEDED.md` file.

**Purpose:** Queue blocking actions instead of stopping work.

**Format:**

```markdown
# Human Actions Needed

## Pending

### [2025-01-14] Run build command
Context: Need to verify proto changes compile
Command: `make protogen`
Blocking: Implementation of match_baton_id for App
After completion: Notify Claude to continue with controller changes

### [2025-01-14] Approve PR #123
Context: Dependency update required for feature
Blocking: Integration testing
After completion: Can proceed with end-to-end tests

## Completed

### [2025-01-13] ~~Get API credentials~~
Completed: 2025-01-14
Result: Credentials added to .env
```

**Rules:**

- Add items immediately when blocked, then continue other work
- Include enough context that work can resume when action completes
- Move completed items to Completed section (don't delete)
- Only ask human for input when ALL completable work is done

**Why:** Maximizes autonomous progress. Respects human time by batching requests.

---

### 4. old/ Directory (when deprecating code)

When superseding code, move it to `old/` directory instead of deleting.

**Structure:**

```
old/
├── README.md           # Index of what's here and why
├── original_auth/      # Deprecated auth implementation
│   ├── README.md       # Why deprecated, what replaced it
│   └── ...
└── v1_api/
    ├── README.md
    └── ...
```

**Each subdirectory README must document:**

- What the code was
- Why it was deprecated
- The antipattern it represents
- What replaced it

**Why:** Creates fossil record for deriving antipatterns. Deletion loses learning opportunities.

---

### 5. DEMO.md (after completing user-facing features)

When code completely satisfies a use case, write a DEMO markdown file.

**Contents:**

- Step-by-step walkthrough of the use case
- Commands to run at each step
- Expected output at each step
- Must be runnable by user following the steps

**Why:** Validates UX makes sense. Serves as executable documentation.

---

## Mandatory Practices

### File Versioning (Not Overwriting)

When receiving feedback or notes on a document:

- DO NOT overwrite the existing file
- Create a new version: `FILENAME_V2.md`, `FILENAME_V3.md`
- Or use phase suffixes: `FILENAME_PHASE2.md`, `FILENAME_REVISED.md`
- Preserve older content for reference

**Exception:** Typo fixes or minor corrections can update in place.

**Why:** Preserves decision history. Enables comparing iterations.

---

### Stable Identifiers (Never Renumber)

Once an item has a number (backlog item, requirement, etc.), it keeps that number forever.

**Rules:**

- When items are removed, keep original numbers with holes: 1, 2, 3, 5, 7, 11
- Mark removed items with ~~strikethrough~~ and note why removed
- When adding items, use the next available number
- Items may be referenced in other documents, commit messages, discussions

**Why:** Renumbering breaks references. External systems may depend on stable IDs.

---

### Checkpoint Commits

**Rules:**

- Create checkpoint commits when meaningful unit of work completes
- Create checkpoint commits before beginning major changes
- When work is organized into phases, commit at end of each phase
- Commit message should reference phase number and summarize accomplishment
- Prefer larger commits for checkpoints (capture coherent state)
- Don't wait to be asked - commit proactively

**Why:** Preserves rollback points. Documents progress. Enables bisection.

---

### Catalog Documents for Enumerable Things

When a project has categories of enumerable items (components, endpoints, commands, etc.):

- Maintain a markdown catalog document listing them all
- Include key metadata for each item
- Catalog is authoritative source, not filesystem
- Update catalog when adding new items to category

**Why:** Enables discovery without filesystem traversal. Single source of truth.

---

### Scripts in Project Directory

All scripts (bash, Python, etc.) created during work MUST be saved in the project directory.

**Rules:**

- Save scripts to `./scripts/` subdirectory in the project root
- Never save scripts to `/tmp/` or other ephemeral locations
- Scripts are project artifacts and should be committed
- Include a brief comment header explaining what the script does

**Format:**

```
project-root/
└── scripts/
    ├── analyze_connectors.sh
    ├── check_coverage.py
    └── ...
```

**Why:** Scripts are intellectual property. They document methodology. Others may need to re-run or modify them.

---

## Self-Check

Before considering any project phase complete:

- [ ] DATA_SOURCES.md exists and is current
- [ ] LEARNINGS.md captures discoveries made
- [ ] HUMAN_ACTIONS_NEEDED.md is empty or only contains truly blocked items
- [ ] Deprecated code moved to old/ with documentation (if applicable)
- [ ] DEMO.md written for user-facing features (if applicable)
- [ ] Documents versioned, not overwritten
- [ ] Checkpoint commit created for phase
- [ ] Catalog documents updated (if enumerable items added)
