# Project Practices

Mandatory practices for all projects.

## File Versioning

When receiving feedback on a document:
- DO NOT overwrite existing file
- Create new version: `FILENAME_V2.md`, `FILENAME_V3.md`
- Or: `FILENAME_PHASE2.md`, `FILENAME_REVISED.md`
- Preserve older content for reference

Exception: Typo fixes can update in place.

## Stable Identifiers

Once an item has a number, it keeps that number forever.

- When items removed, keep holes: 1, 2, 3, 5, 7, 11
- Mark removed with ~~strikethrough~~ and note why
- New items use next available number

Why: Items may be referenced in docs, commits, discussions.

## Checkpoint Commits

- Create when meaningful unit completes
- Create before major changes
- When work has phases, commit at end of each phase
- Message references phase and summarizes accomplishment
- Prefer larger commits (capture coherent state)
- Don't wait to be asked

## Catalog Documents

For categories of enumerable items (components, endpoints, commands):
- Maintain markdown catalog listing them all
- Include key metadata for each
- Catalog is authoritative source, not filesystem
- Update catalog when adding new items

## "Show Me the Code"

All architectural analysis MUST be grounded in actual code.

- Don't theorize - read the implementation
- Don't assume bottlenecks - find them in code
- Don't claim limitations without citing files and line numbers

Bad: "The current system probably does X"
Good: "In pkg/access/resolver.go:234, the query iterates over all grants - O(n)"

## Subjunctive = "Do It If Possible"

When notes use "perhaps", "could", "might", "if feasible":

| Written | Interpretation |
|---------|----------------|
| "perhaps using Z3" | Implement Z3 if possible |
| "could add caching" | Add caching unless blocked |
| "might support X" | Support X if feasible |

Subjunctive = exploratory scope, NOT optional scope.
Only skip if technically impossible.

## Test Generated Code Compiles

Any code generation (scaffolding, templates, codegen) MUST have tests that:

1. Generate output
2. Run the build tool (go build, npm build, etc.)
3. Fail if build fails

Without this, generated code drifts from reality silently.

**Why:** SDK APIs evolve. Templates written for SDK v1 break on SDK v2.
Discovered when cone connector init generated code that didn't compile.

**Applies to:**
- CLI scaffolding commands (init, create, new)
- Cookiecutter/yeoman/create-X templates
- Code generators (protoc, openapi-generator, etc.)
- Any tool that outputs code meant to be built

## Ticketing and Wiki Integration

When ticketing system or wiki integrations are available:

**On project start or when working on a project without existing representation:**
1. Check if the project already has an associated issue or wiki page
2. If not, ask the user whether to:
   - Create a new issue/page for this project
   - Attach to an existing one (provide a list of candidates)
3. Query the integration for relevant existing items to offer as attachment options

**Why:** Projects should be tracked in the system of record. Asking early prevents orphaned work and ensures visibility.

**What to list:**
- For ticketing: Recent issues in the relevant team/project, issues with related labels
- For wiki: Recent pages in the relevant space/database, pages with related tags

## Upstream/Downstream Agent Communication

Projects can receive goals and data sources from upstream agents (agents working in other projects that feed into this one).

### INBOX Directory

Every project MAY have an `INBOX/` directory at its root:

```
project/
├── INBOX/
│   ├── README.md  # MUST specify which agent reads from this INBOX
│   ├── goal_001_axiomatize_connectors.md
│   ├── goal_002_verify_okta_model.md
│   ├── datasource_001_connector_catalog.md
│   └── processed/
│       └── goal_000_initial_setup.md
```

**INBOX README Requirement**: Each INBOX directory MUST have a README.md that specifies exactly one agent who reads from that INBOX. This prevents confusion when multiple agents operate in the same repository. Example:

```markdown
# INBOX

**Reader:** VERIFY

This INBOX is read by the VERIFY agent. Other agents should NOT process items here.
```

### Rules for Upstream Agents

1. **Append-only**: Never modify or delete existing INBOX files
2. **One item per file**: Each file is a single goal OR data source
3. **Naming**: `goal_NNN_short_description.md` or `datasource_NNN_short_description.md`
4. **Format**:

```markdown
# Goal: Short Title
<!-- or # Data Source: Short Title -->

**From**: project-path or agent-id
**Priority**: high | medium | low
**Added**: YYYY-MM-DD HH:MM

## Description

What needs to be done or what data is being provided.

## Context

Why this matters, how it relates to upstream work.

## Acceptance Criteria (for goals)

- [ ] Specific measurable outcome
- [ ] Another outcome
```

### Rules for Downstream Projects

1. **Check INBOX on start**: When beginning work, check for new items
2. **Process items**: Add goals to TODO, add data sources to DATA_SOURCES.md
3. **Mark processed**: Move to `INBOX/processed/` or prefix filename with `PROCESSED_`
4. **Never delete**: Keep for audit trail

### Why This Exists

- Upstream agents may have context the downstream agent lacks
- Enables pipeline-style multi-agent workflows
- Creates traceable provenance for goals
- Allows async communication between agents

### Example Workflow

1. Agent A (working in `research/analysis`) discovers that `research/spike-classifiers` needs new axioms
2. Agent A creates `spike-classifiers/INBOX/goal_042_auth0_axioms.md`
3. Later, Agent B starts work in `spike-classifiers`
4. Agent B checks INBOX, sees new goal, adds to task list
5. Agent B moves file to `INBOX/processed/` after completing

### Neighbor Subproject Requests

In multi-subproject repos, sibling subprojects can request work from each other.

Example: `18-go-afl` discovers it needs a new classifier operation from `01-classifiers`:

```
spike-classifiers/
├── 01-classifiers/
│   └── INBOX/
│       └── goal_003_add_powerset_operation.md
├── 18-go-afl/
│   └── (requesting subproject)
```

Rules for neighbor requests:
1. **Same INBOX format**: Use standard goal/datasource format
2. **Include requester**: Add `**From**: ../18-go-afl` or similar relative path
3. **Cross-reference**: Requesting subproject notes the request in its own TODO/LEARNINGS
4. **No circular dependencies**: If A requests from B, B should not need A to complete the request

When to use neighbor requests vs doing it yourself:
- **Request**: The work belongs in the neighbor's domain/responsibility
- **Do yourself**: It's integration code that spans both (put in integration layer)
- **Request**: You lack context about the neighbor's internals
- **Do yourself**: It's a trivial addition you fully understand

## Multi-Agent Coordination (COORDINATOR Protocol)

When multiple agents work in the same repository:

### COORDINATOR INBOX

The repo root has an `INBOX/` directory for agent-to-COORDINATOR communication:

```
repo/
├── INBOX/
│   ├── README.md           # Protocol documentation
│   ├── report_ALPHA_*.md   # Progress reports
│   ├── request_BETA_*.md   # Requests/questions
│   └── blocked_GAMMA_*.md  # Blocker notifications
```

### Pending Work on Context Compaction

**Critical:** Report pending work to COORDINATOR BEFORE compaction, not after.

When you notice context is getting full or receive a compaction warning:

1. **Immediately** drop a progress report to COORDINATOR INBOX
2. Include:
   - What you're currently working on (in progress)
   - What remains to be done
   - Any context the next session will need
   - Files modified but not committed
3. This ensures the next session can resume without information loss

**Why before, not after:**
- After compaction, the new session has no memory of pending work
- The progress report becomes the source of truth
- COORDINATOR can reassign or queue the work appropriately

### Role Assignments

Agents receive assignments via their project's INBOX:
- `<subproject>/INBOX/assignment_<ROLE>_*.md`

The assignment defines:
- Your role name (e.g., ALPHA, BETA, DELTA)
- Your territory (files you own)
- Read-only areas
- Do-not-touch areas
- Current tasks

### Protocol Requirements

1. **Prefix messages**: `ROLE: <message>` (e.g., `DELTA: Task complete`)
2. **Progress reports**: At task completion, phase boundaries, session end
3. **Stay in territory**: Don't edit files outside your assigned directories
4. **Report blockers**: Immediately, not at scheduled reports
5. **Coordinate on conflicts**: If you touched another agent's territory, coordinate via their INBOX

## Self-Check

Before any phase complete:
- [ ] DATA_SOURCES.md current
- [ ] LEARNINGS.md captures discoveries
- [ ] HUMAN_TODOS.md empty or truly blocked
- [ ] Deprecated code in old/ with docs
- [ ] Sensitive docs in private/
- [ ] DEMO.md for user-facing features
- [ ] Documents versioned, not overwritten
- [ ] Checkpoint commit created
- [ ] Catalog documents updated
- [ ] INBOX/ checked and processed
- [ ] If multi-agent: Progress report to COORDINATOR
