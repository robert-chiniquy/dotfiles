# Project Artifacts

Mandatory files for projects. Create these in project root.

## DATA_SOURCES.md

Track provenance of all information.

```markdown
# Data Sources

## Filesystem
- `/path/to/repo/` - What was learned
- `/path/to/file.go:123-456` - Specific code referenced

## URLs
- https://example.com/docs - Content description

## Other
- [2025-01-14] Slack conversation - Key insight
```

Rules:
- Add sources as consulted, not retroactively
- Include line numbers for code references
- Date-stamp non-permanent sources

Bad: "from code analysis"
Good: `/Users/rch/repo/baton-okta/pkg/connector/app.go:334-336`

## LEARNINGS.md

Preserve discoveries with dated headers.

```markdown
# LEARNINGS.md

## 2025-01-14: Topic Name

What was learned, why it matters.
- Concrete examples
- File paths with line numbers
- Code snippets
```

Rules:
- Add immediately when discovered
- Append-only (never delete)
- Include enough detail to reconstruct reasoning

## HUMAN_TODOS.md

Queue blocking actions instead of stopping.

```markdown
# Human Actions Needed

## Pending

### [2025-01-14] Run build command
Context: Need to verify proto changes compile
Command: `make protogen`
Blocking: Implementation of match_baton_id
After completion: Continue with controller changes

## Completed

### [2025-01-13] ~~Get API credentials~~
Completed: 2025-01-14
Result: Credentials added to .env
```

Rules:
- Add items immediately when blocked
- Continue other work
- Move completed to Completed section
- Only ask human when ALL work is blocked

## old/ Directory

Move superseded code here instead of deleting.

```
old/
├── README.md           # Index of what's here
├── original_auth/
│   ├── README.md       # Why deprecated, what replaced it
│   └── ...
```

Each README documents:
- What the code was
- Why deprecated
- The antipattern it represents
- What replaced it

## private/ Directory

Documents too sensitive to share externally.

```
private/
├── README.md           # Why each file is private
├── SENSITIVE_DOC.md
```

Criteria (any of these):
- Overpromises or unrealistic expectations
- Exposes weaknesses without context
- Could be read as critical
- Harsh language about own work
- Alarming assessments without context
- Stale claims not updated

Rule: Can always move to public later, can't unpublish.

## PLAN_*.md

All plans written to local files.

Naming: `PLAN_<SPECIFIC_OBJECTIVE>.md`
- `PLAN_IMPLEMENT_DFA_CACHING.md`
- `PLAN_DATALOG_BENCHMARK.md`

Contents:
- Objective
- Context
- Approach
- Steps
- Success criteria
- Open questions

Rules:
- Write before implementation
- Multiple plans can coexist
- Append-only during execution
- Mark completed/abandoned but preserve
- Large projects: put plans in `plans/` subdirectory to reduce top-level clutter

## FAILURES.md

Document what proved impossible.

```markdown
# FAILURES.md

## [2025-01-17] Attempted: Real-time DFA minimization

**Goal:** Minimize DFAs during product construction

**What we tried:**
- Hopcroft's algorithm mid-construction
- Incremental state merging

**Why it failed:**
- Intermediate states may become reachable later
- Minimization only valid for complete DFAs

**Lesson:** Minimization must wait until DFA is complete.
```

Rules:
- Add immediately when something proves impossible
- Include what was tried and why it failed
- Document alternatives considered
- Append-only

## DEMO.md

Write after completing user-facing features.

Contents:
- Step-by-step walkthrough
- Commands at each step
- Expected output
- Must be runnable by user
