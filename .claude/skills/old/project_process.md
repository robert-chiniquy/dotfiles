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

### 3. HUMAN_TODOS.md (when applicable)

When work requires human action that blocks progress, maintain a `HUMAN_TODOS.md` file.

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

### 5. private/ Directory (for sensitive documents)

Documents that shouldn't be shared externally should live in a `private/` directory, excluded from git.

**Purpose:** Protect documents that could be misread, misinterpreted, or are not yet ready for external audiences.

**Structure:**

```
private/
├── README.md           # Index explaining why each file is private
├── SENSITIVE_DOC.md    # Harsh internal critique
└── ...
```

**The README must explain for each file:**

- What the document contains
- Why it's private (specific concern)

**Criteria for private/ (any of these):**

1. **Overpromises or unrealistic expectations** - Early drafts that promise more than can be delivered
2. **Exposes internal weaknesses without context** - Product gaps, limitations that need framing
3. **Could be read as critical of team/product/customers** - Even if accurate, needs careful positioning
4. **Harsh language about our own work** - Internal critiques meant for improvement
5. **Alarming assessments without context** - Risk documents, "NOT READY" statuses
6. **Stale claims that haven't been updated** - Earlier versions with outdated information

**Workflow:**

1. Create document normally during work
2. Before sharing/publishing, review against criteria above
3. If ANY criteria match, move to private/
4. Add entry to private/README.md
5. Can always move back to public later, but can't unpublish

**Why:** You can't unpublish. Better to keep things private and add later than publish and regret. Protects team from documents that are accurate but could be misinterpreted without context.

**Relationship to old/:**

- `old/` = superseded code/docs preserved for learning (can be public)
- `private/` = documents too sensitive to share (excluded from git)

---

### 6. PLAN_*.md (for all plans)

All plans of any kind MUST be written to local plan markdown files in the project directory.

**Naming convention:** `PLAN_<SPECIFIC_OBJECTIVE>.md`

**Examples:**
- `PLAN_IMPLEMENT_DFA_CACHING.md`
- `PLAN_DATALOG_BENCHMARK_INTEGRATION.md`
- `PLAN_SMT_PROOF_GENERATION.md`

**Rules:**
- Write the plan BEFORE implementation
- Use descriptive objective names (not generic like `PLAN_PHASE1.md`)
- Multiple plans can coexist (different objectives)
- List all active plans in PROJECT.md if it exists
- Plans are append-only during execution (don't delete steps that proved wrong)
- Mark completed/abandoned sections but preserve them

**Contents:**
- Objective: What are we trying to achieve?
- Context: What do we know? What constraints exist?
- Approach: How will we achieve it?
- Steps: What are the concrete steps?
- Success criteria: How will we know it's done?
- Open questions: What needs clarification?

**Why:** Plans are artifacts. They document reasoning. Future sessions can resume from them. Written plans prevent scope creep and forgotten requirements.

---

### 7. FAILURES.md (when something proves impossible)

When a task or feature proves permanently impossible, document it in `FAILURES.md`.

**Purpose:** Track what we tried and why it didn't work. Prevents re-attempting failed approaches.

**Format:**

```markdown
# FAILURES.md

## [2025-01-17] Attempted: Real-time DFA minimization

**Goal:** Minimize DFAs during product construction

**What we tried:**
- Hopcroft's algorithm mid-construction
- Incremental state merging

**Why it failed:**
- Intermediate states may become reachable later
- Minimization is only valid for complete DFAs

**Alternatives considered:**
- Lazy minimization (implemented instead)
- BDD representation

**Lesson:** Minimization must wait until DFA is complete.
```

**Rules:**
- Add immediately when something proves impossible
- Include what was tried and why it failed
- Document alternatives considered
- Extract lessons learned
- Never delete entries (append-only)

**Why:** Prevents wasted effort on known dead ends. Captures institutional knowledge.

---

### 8. DEMO.md (after completing user-facing features)

When code completely satisfies a use case, write a DEMO markdown file.

**Contents:**

- Step-by-step walkthrough of the use case
- Commands to run at each step
- Expected output at each step
- Must be runnable by user following the steps

**Why:** Validates UX makes sense. Serves as executable documentation.

---

## Interpreting Project Requirements

### "Show Me the Code" is a Premise

All architectural analysis, critique, and design work MUST be grounded in actual code examination.

**Rules:**
- Don't theorize about what a system does - read the implementation
- Don't assume bottlenecks - find them in the code
- Don't claim limitations without citing specific files and line numbers
- Reference actual data structures, algorithms, and query patterns
- When analyzing a system you're improving upon, read that system's code first

**Anti-patterns:**
- "The current system probably does X" (speculation)
- "This would be slow because Y" (assumption without evidence)
- "The architecture likely has Z problem" (theory without code)

**Good patterns:**
- "In pkg/access/resolver.go:234, the query iterates over all grants - O(n)"
- "The GroupMembership struct at models/group.go:45 doesn't support transitive closure"
- "baton-entra/pkg/connector/users.go:178 fetches all users then filters client-side"

**Why:** Code is ground truth. Documentation lies. Assumptions compound. Reading code prevents building solutions to imaginary problems.

---

### Subjunctive Mood = "If Possible, Do It"

When project notes, plans, or requirements use subjunctive mood ("perhaps", "could", "might", "if feasible"), interpret this as:

**"Do everything that is possible. Do not stop at the minimum."**

| Written | Interpretation |
|---------|----------------|
| "perhaps using Z3" | Implement Z3 integration if possible |
| "could add caching" | Add caching unless blocked |
| "might support X" | Support X if feasible |
| "explore whether Y" | Implement Y if research shows it's possible |

The subjunctive indicates exploratory scope, NOT optional scope. The work is only skipped if it proves technically impossible or permanently blocked.

**Do not interpret subjunctive as:**
- "Nice to have"
- "Only if there's time"
- "Lower priority"

**Correct interpretation:**
- "Required unless blocked"
- "Maximum scope unless impossible"
- "Intent to implement, contingent on feasibility"

**Example:** A project spec saying "perhaps using z3 or another SMT approach" means: Implement Z3 integration. Only stop if Z3 proves fundamentally incompatible with the architecture.

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

### Meta Project Makefiles

Multi-subproject repositories should have a root Makefile that propagates targets to subprojects.

**CRITICAL: When adding new subprojects or targets, ALWAYS update the root Makefile:**

1. **New subproject** -> Add targets: `<subproject>-build`, `<subproject>-test`, `<subproject>-clean`
2. **New target in subproject Makefile** -> Add corresponding target in root Makefile
3. **Add to aggregate targets** -> Include in `build`, `test`, `clean` that iterate over all subprojects

Example when adding `09-integration`:
```makefile
# Add to SUBDIRS
SUBDIRS = 01-classifiers 02-egraph ... 09-integration

# Add specific targets
09-integration-build:
	$(MAKE) -C 09-integration build

09-integration-test:
	$(MAKE) -C 09-integration test
```

This ensures `make build` and `make test` from root include all subprojects.

**Argument passthrough:**

The root Makefile MUST support passing arguments through to underlying commands:

```bash
# Pass -v flag to all go test commands
make test -- -v

# Pass -vvvv or any flags
make test -- -vvvv

# Build with specific flags
make build -- -race
```

**Implementation pattern:**

```makefile
# At top of Makefile, capture extra args
ARGS = $(filter-out $@,$(MAKECMDGOALS))

# Or use this pattern for -- separation
%:
	@:

test:
	@for dir in $(SUBDIRS); do \
		$(MAKE) -C $$dir test EXTRA_FLAGS="$(ARGS)"; \
	done

# In subproject Makefile
test:
	go test $(EXTRA_FLAGS) ./...
```

**Why:** Developers expect to pass flags through. `-v` for verbose, `-race` for race detection, `-count=1` to disable caching.

---

### Project Organization: Status Subdirectories

Large projects MUST use status subdirectories to keep the top-level project focused on currently operating specs.

**Required structure:**
```
project/
├── old/           # Superseded versions (V1 when V2 exists, etc.)
├── design/        # Design documents (DESIGN_*.md, *_DESIGN.md)
├── plans/         # Plans (PLAN_*.md, *_PLAN.md)
├── analysis/      # Analysis, research, axioms, assessments
├── reference/     # Reference documentation (concepts, guides, inventories)
├── retro/         # Retrospectives, critiques, failures, lessons learned
└── [process docs] # Only active process files at top level
```

**Top-level files (keep at root):**
- CLAUDE.md, README.md, project.md - Project definition
- DATA_SOURCES.md, GLOSSARY.md, LEARNINGS.md - Knowledge tracking
- TODO.md, COMPLETED.md, HUMAN_TODOS.md - Task tracking
- DEMO.md or DEMO_V2.md - Current showcase

**Categorization rules:**
- **old/**: Any file with V1, V2, etc. when a newer version exists. Also: redundant summaries, temporary inventories, superseded approaches.
- **design/**: DESIGN_*.md, *_DESIGN.md, *_INTEGRATION_DESIGN*.md, UI designs
- **plans/**: PLAN_*.md, *_PLAN.md, implementation roadmaps
- **analysis/**: *_ANALYSIS.md, *_AXIOMATIZATION*.md, *_ASSESSMENT.md, scale analysis, product analysis
- **reference/**: Concept explanations, how-to guides, feature inventories, benchmarks, protocol diffs
- **retro/**: FAILURES.md, *_CRITIQUE.md, *_GAPS.md, SUGGESTIONS.md, process improvements, verification audits

**Periodic cleanup (every ~50 files at top level):**
1. List all top-level .md files
2. Check STATUS header or infer from content/naming
3. Move to appropriate subdirectory
4. Update any cross-references

**Why:** Prevents clutter. Makes it clear what's current vs historical. Large projects accumulate documents quickly; this keeps focus on what matters now.

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

### Design Before Implementation (TDD Principle)

Design tasks always take precedence over implementation tasks. Following TDD principles: design for tests comes before design for implementation.

**Every design task completed is an implementation task you have not yet begun.**

This is a reminder: design documents create work, they don't complete it. A design doc for feature X is valuable, but feature X remains unimplemented until code exists. Don't mistake planning for progress.

**Order of priority:**
1. New data sources and research topics (highest)
2. Design and planning tasks
3. Test design (what should be tested, how to verify correctness)
4. Implementation tasks (lowest)

**TDD workflow:**
1. **Design** - What problem are we solving?
2. **Test design** - How will we know it's correct? What properties should hold?
3. **Implementation** - Write code that passes the tests

**Rationale:** Implementation without design leads to rework. Tests without design verify the wrong things. Good design reduces total effort. But design without implementation is just documentation.

**Signals to pause implementation:**
- Uncertainty about approach
- Multiple valid solutions
- Missing requirements
- New information that changes assumptions
- Tests not yet designed

---

### New Data Sources and Research Topics Get Priority

When a new data source arrives or a new research topic opens up:

1. **Immediately promote it to highest priority** in the current work queue
2. **Investigate before continuing other work** - new information may change decisions
3. **Update DATA_SOURCES.md** with the new source
4. **Document findings in LEARNINGS.md** before moving on

**Rationale:** New data sources often contain insights that affect other work. Investigating first prevents wasted effort on invalidated assumptions.

**Example:**
- User mentions "there's a 1.1GB c1z file under ~/repo" -> find it, examine it, update plans
- User suggests "look at the RAP implementation in research/agents/" -> read it before designing prompts interface
- User asks "have you checked the original classifiers repo for caching?" -> search there first

---

### Maintain Momentum (Don't Block on Human)

When work requires human action:

1. **Add the needed action to HUMAN_TODOS.md**
2. **Immediately continue with other available work**
   - Other threads/tasks
   - Design docs
   - Implementation plans
   - Tests
   - Documentation
3. **Only ask human for input when ALL completable work is done**

**Rules:**

- Don't stop and wait for approval unless completely blocked on everything
- Batch requests rather than blocking repeatedly
- If uncertain about a decision, make a reasonable choice and document it
- Queue permissions and approvals for human review
- Continue with unblocked work in parallel

**Anti-pattern:** Stopping work to ask "should I proceed?" or "is this okay?"

**Good pattern:** "I've queued these items in HUMAN_TODOS.md and continued with the remaining tasks."

**Why:** Maximizes autonomous progress. Respects human time. Keeps momentum.

---

## Meta-Learnings: Complex Multi-Subproject Management

Patterns extracted from managing complex research spikes with multiple interrelated subprojects.

### Build System Discipline

**Never bypass the root Makefile in multi-subproject repos.**

| Wrong | Right |
|-------|-------|
| `cd subproject && go test ./...` | `make subproject-test` |
| `cd subproject && make test` | `make subproject-test` |
| `go build ./subproject/...` | `make subproject-build` |

Direct commands may hang indefinitely due to toolchain configuration handled by the root Makefile. This is especially common with Go modules in monorepos.

**The root Makefile MUST:**
- Propagate all targets to subprojects
- Handle environment setup correctly
- Support argument passthrough (`make test -- -v`)

If commands are timing out or hanging, the FIRST thing to check is whether you're using the root Makefile.

---

### Context Compaction Recovery

**Critical information must appear in multiple locations because context compaction loses memory.**

After compaction, Claude Code starts fresh. Important rules must be:
1. In `CLAUDE.md` at the TOP (read first after compaction)
2. In `LEARNINGS.md` prominently
3. Repeated in both files, not just one

**Pattern:** Add a "READ THIS FIRST" section at the top of CLAUDE.md:
```markdown
## READ THIS FIRST AFTER EVERY CONTEXT COMPACTION

[Critical rules that MUST be followed]
```

If you've been reminded multiple times about the same mistake:
1. Add it to CLAUDE.md immediately
2. Make it prominent (top of file)
3. Also add to LEARNINGS.md

---

### Existing Code Discovery

**Always search existing codebase before assuming something is missing.**

Before implementing tests/features:
1. Search for files with related naming conventions (`*_test.go`, `laws_*.go`)
2. Check for alternative naming patterns (`laws_` vs `test_`)
3. Grep for function names you expect to exist

**Example mistake:** Assuming no property-based tests existed because they used `laws_*.go` naming instead of `*_test.go`.

**Good patterns:**
```bash
# Find all test files
make list-tests  # If available
glob **/*test*.go

# Find related implementations
grep -r "func.*Property" --include="*.go"
```

---

### Unifying Algorithms

**When implementing multiple related operations, find the unifying algorithm.**

Example: All set operations (Union, Intersection, Difference, Complement, SymmetricDifference) can use the same product construction algorithm with different "painter" functions:

| Operation | Painter Logic |
|-----------|--------------|
| Union | accept if either accepts |
| Intersection | accept if both accept |
| Difference | accept if left accepts, right doesn't |
| SymmetricDifference | accept if exactly one accepts |

One algorithm, many operations. This reduces bugs and maintenance burden.

---

### Avoiding Mutual Recursion

**When implementing derived predicates, ensure the call graph is acyclic.**

Dangerous pattern:
```go
func (a *TypeA) Equals(b *TypeA) bool {
    return a.SymmetricDifference(b).IsEmpty()  // Calls B.method()
}

func (b *TypeB) IsEmpty() bool {
    return b.left.Equals(b.right)  // Calls A.Equals() -> infinite loop!
}
```

**Solutions:**
1. Compute derived predicates via a DIFFERENT mechanism (e.g., DFA product construction)
2. Use conservative approximations when exact computation would recurse
3. Separate "structural" operations from "derived" predicates

---

### Conservative Approximation

**When exact computation is expensive or impossible, return conservative estimates.**

Examples:
- `Relation()` returning `RelationIntersection` ("might overlap") instead of precise relation
- `IsEmpty()` returning `false` when unable to determine emptiness
- `IsSubsetOf()` returning `false` when unable to verify

Mark these clearly in documentation. Conservative approximations are SAFE (won't cause incorrect behavior) but may miss optimization opportunities.

---

### Cross-Validation Testing

**Different representations of the same concept can verify each other.**

When you have multiple implementations (DFA, Datalog, SMT, native):
1. Generate random test cases
2. Evaluate with each representation
3. Assert all results agree
4. On disagreement, minimize to find bug

This catches bugs that single-representation tests miss. Particularly valuable for complex domains like access control.

---

### Subproject Isolation

**Each subproject in a spike should be independently usable.**

Requirements:
- Own `go.mod` (or equivalent)
- Own tests (runnable via root Makefile)
- Own CLAUDE.md if it has code
- Own DATA_SOURCES.md if it consulted external sources
- Clear interface to other subprojects

Dependencies between subprojects should be explicit (replace directives during dev, proper module paths for release).

---

### Separation of Concerns

**It is better to create a new subproject than to pollute an existing one.**

When adding functionality that spans multiple concerns:
1. Ask: "Does this belong in the existing subproject's single responsibility?"
2. If NO: Create a new subproject for the integration/bridge layer
3. If YES: Add it, but watch for growing scope

**Signs a subproject needs splitting:**
- Import cycles between packages within the subproject
- Tests that require mocking unrelated concerns
- Multiple independent "main" entry points
- Different deployment or consumption patterns for different parts

**Integration layers belong in their own subprojects:**
- `classifiers-egraph-bridge/` not `classifiers/egraph.go`
- `datalog-smt-bridge/` not `datalog/smt.go`
- `integration/` for unified services that combine multiple subprojects

**Why:** Clean boundaries enable independent evolution. Coupling at integration points is explicit and manageable. Each subproject can be understood, tested, and used in isolation.

---

## Self-Check

Before considering any project phase complete:

- [ ] DATA_SOURCES.md exists and is current
- [ ] LEARNINGS.md captures discoveries made
- [ ] HUMAN_TODOS.md is empty or only contains truly blocked items
- [ ] Deprecated code moved to old/ with documentation (if applicable)
- [ ] Sensitive documents in private/ with README entry (if applicable)
- [ ] DEMO.md written for user-facing features (if applicable)
- [ ] Documents versioned, not overwritten
- [ ] Checkpoint commit created for phase
- [ ] Catalog documents updated (if enumerable items added)
- [ ] Root Makefile tested for all subprojects (multi-subproject repos)
