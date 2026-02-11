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

## TODO.md

Track work items and blocking actions.

```markdown
# TODO

- [ ] Run build command (`make protogen`) to verify proto changes
- [ ] Get API credentials for .env
```

Rules:
- Add items immediately when blocked
- Continue other work
- Check off completed items
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

## Git Commit Policy

Project meta-documentation has different commit rules based on project type.

### Shared Codebases (has git remote)

These files must NEVER be committed:
- `DATA_SOURCES.md`
- `LEARNINGS.md`
- `GLOSSARY.md`
- `TODO.md`
- `PLAN_*.md`
- `.claude/plans/`

Setup global gitignore to prevent accidents:

```bash
# Create global gitignore
cat >> ~/.gitignore_global << 'EOF'
# Project meta-documentation (local-only, never commit)
PLAN_*.md
DATA_SOURCES.md
LEARNINGS.md
GLOSSARY.md
TODO.md
.claude/plans/
EOF

# Configure git to use it
git config --global core.excludesFile ~/.gitignore_global
```

When initializing in a shared codebase:
1. Check for git remote: `git remote -v`
2. If remote exists, add files to project's `.gitignore`
3. Inform user: "These meta-documents are local-only and will not be committed"

### Private Projects (no git remote)

If a project has no git remote when started, meta-documentation CAN be committed.

Check at init:
```bash
git remote -v  # Empty output = no remote
```

When no remote exists:
- These files become part of the project
- Commit them normally
- If a remote is added later, decide then whether to keep or gitignore

This allows personal/private projects to version-control their learnings.

## Upstream/Downstream Projects

Projects may have relationships across repos.

### Upstream Projects

Projects that provide designs, requirements, or goals to downstream projects.

Visibility levels:
- **Private**: Not shared with other humans (but may be shared with agents)
- **Secret**: Not shared with other agents either (user mediates all information)

Rules:
- Upstream projects are sometimes secret or private
- Never ask about upstream project contents if not provided
- Designs flow downstream; the user mediates what information transfers
- If upstream is secret, you receive only the outputs (goals, designs, specs)
- Don't probe for details about secret upstream projects

### Downstream Projects

Projects that implement designs from upstream.

Rules:
- Wait for input from upstream before starting implementation
- Goals are added to downstream project for execution
- Downstream agent works from goals without needing upstream context
- Document in DATA_SOURCES.md: "Design from upstream project (private)"

### Information Flow

```
[Upstream Project] ---(user mediates)---> [Downstream Project]
     (may be secret)                        (implements)
```

The user controls what crosses the boundary. Don't probe for upstream details.

## sources/ Directory

Cache external artifacts locally for offline development and study.

```
sources/
├── README.md           # Index of cached artifacts
├── papers/
│   ├── chow-ruskey-2012.pdf
│   └── speuler-vis2021.pdf
├── specs/
│   └── rfc-7519-jwt.txt
└── snapshots/
    └── api-docs-2025-01-28.html
```

Purpose:
- Enable offline development without internet
- Preserve artifacts that may disappear (URLs go stale)
- Allow human study of reference materials
- Version-pin external dependencies

README.md should include:
- Original URL for each artifact
- Date downloaded
- Brief description of contents
- Which DATA_SOURCES.md entries reference it

Rules:
- Add to .gitignore if artifacts are large or have redistribution concerns
- Prefer PDFs over HTML (more stable)
- Include version/date in filename when content may change
- Reference local paths in DATA_SOURCES.md after caching
