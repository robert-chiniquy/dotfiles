# Preferences

- Write all code to files (even temporary scripts) for tracking
- Ask clarifying questions when scope, approach, or requirements are ambiguous
- Never add project-specific or repo-specific notes to this global config; those belong in each project's `.claude/CLAUDE.md`
- **ABSOLUTELY NO EMOJI EVER** - This is a hard rule with ZERO exceptions unless user explicitly instructs "use emoji". Emoji includes but is not limited to:
  - Unicode emoji characters (✅ ❌ 🚧 ⚠️ 📝 etc.)
  - Emoticons :-) :( ;-) etc.
  - Symbol characters used decoratively (★ ✓ ✗ ► • ◆ etc.)
  - Any Unicode character outside basic ASCII printable set when used for visual decoration
  - This applies to ALL output: prose, code comments, commit messages, documentation, tables, lists, status indicators
  - Use text instead: DONE/COMPLETE/PASS instead of ✅, TODO/PENDING instead of 🚧, FAIL/ERROR instead of ❌
  - When scanning your output before sending, actively check for any Unicode decorative characters and replace them with plain text
  - If you find yourself wanting visual emphasis, use **bold**, *italic*, CAPS, or structural formatting (headings, lists) instead
- **When user provides "always" guidance** - Immediately add it to this global config to ensure it persists forever across all sessions
- **"Out of scope" is good** - Don't interpret literally; it means "not doing for now" and we should be happy to reduce scope
- **NEVER estimate human effort** - No effort estimates, no timeline predictions, no person-week calculations unless explicitly asked. This rule overrides ALL skills and methodologies that suggest including time/effort estimates (e.g., systematic_feature_design.md step 10). Omit timing from implementation phases entirely. What to include instead: dependencies (what must be done first), components affected (what systems need changes), complexity indicators (Low/Medium/High if needed).
- **File versioning, not overwriting** - When receiving feedback or notes on a document, DO NOT overwrite the existing file. Create a new version: `FILENAME_V2.md`, `FILENAME_V3.md`, or `FILENAME_PHASE2.md`, `FILENAME_REVISED.md`. Preserve older content for reference. Exception: Typo fixes or minor corrections can update in place.
- **Never renumber backlog items** - Once an item has a number, it keeps that number forever. When items are removed, keep original numbers with holes in the sequence (1, 2, 3, 5, 7, 11). Mark removed items with ~~strikethrough~~ and note why removed. When adding items, use the next available number. Items may be referenced in other documents, commit messages, discussions - renumbering breaks those references.
- **Simple English over business-speak** - Never use business jargon. Banned terms: ROI (use "value" or "benefit vs cost"), KPI (use "metrics"), TCO (use "maintenance cost"), synergy (describe what happens), leverage (use "use"), action item (use "task"), circle back (use "revisit"), low-hanging fruit (use "easy win"), move the needle (use "improve"), bandwidth (use "time" or "capacity"). Business-speak obscures meaning.
- **Show, don't label aesthetics** - When implementing a visual style (vaporwave, retro, etc.), never use the style name in the output itself. The aesthetic should speak through colors, shapes, and design - not by announcing itself. "VAPORWAVE SYSTEM" is cringe; hot pink and cyan gradients are not.
- **Project accent colors via .envrc** - When creating or updating a project's `.envrc`, include a `PROMPT_ACCENT` export using a color from the vaporwave palette. This gives each project a distinct visual identity in the prompt. Palette options:
  - `#ff0099` (hot pink)
  - `#5cecff` (cyan)
  - `#ff00f8` (magenta)
  - `#fbb725` (gold)
  - `#aa00e8` (purple)

  Example: `export PROMPT_ACCENT="#ff0099"`

  Choose colors that feel right for the project's character - cyan for infrastructure, pink for user-facing, gold for data/analytics, purple for experimental.
- **Learning preservation** - When you figure something out, add it to `LEARNINGS.md` in the project root immediately. Document what you learned, why it matters, how you discovered it. Include concrete examples, file paths, code snippets. Use timestamped headers with minute precision (## 2025-01-14 09:45: Topic) for organization. This creates a knowledge base that persists beyond individual conversations. The minute-level precision helps trace when learnings occurred relative to other events.
- **No Co-Authored-By trailers** - Never add Co-Authored-By, Signed-off-by, or similar trailers to git commits.
- **Preserve deprecated code in `old/` directory** - Instead of deleting superseded code, move it to `old/` at project root. Include a README.md documenting:
  - What the code was
  - Why it was deprecated
  - The antipattern it represents
  - What replaced it
  
  This creates a fossil record for deriving antipatterns (e.g., for protogen_stack.md "Common Pitfalls"). Deletion loses learning opportunities.
- **Minimize context pollution from verbose commands** - For commands with large output (protogen, builds, test suites), use `| tail -n 20` or similar to capture only the end. If errors occur, expand as needed. Full verbose output wastes context window on noise.
- **Trace execution chains before adding components** - Before adding a component (function, struct, provider, handler), trace its dependencies through the execution chain. What does it call? What does that need? Follow the chain until you reach things already present. Verify each dependency is satisfied. This applies to wire providers, constructor parameters, interface implementations, and any code that assumes other code exists. Miss one link and the chain breaks at runtime or compile time.
- **Weird Makefile rule** - Commands of the pattern `cd directory && command` are very likely to timeout due to a Claude Code bug. This affects ANY binary, not just specific tools. Instead, either: (1) use the command's built-in directory flag if available (e.g., `git -C /path/to/repo status` instead of `cd /path/to/repo && git status`), or (2) ensure there's a Makefile target that does the thing (e.g., `make web/check` instead of `cd web && npx tsc --noEmit`). The bug causes background tasks and chained cd commands to fail silently or timeout.
- **Prefer larger commits for checkpoint commits** - When committing accumulated work, include all related changes rather than splitting into focused micro-commits. Checkpoint commits capture coherent project state.
- **Proactive checkpoint commits** - Create checkpoint commits when a meaningful unit of work is complete or before beginning major changes. Don't wait to be asked. This preserves rollback points and documents progress.
- **Every phase gets a checkpoint commit** - When work is organized into phases (Phase 1, Phase 2, etc.), always create a checkpoint commit at the end of each phase before moving on. The commit message should reference the phase number and summarize what was accomplished.
- **Automate browser testing with Chrome tools** - When the Claude Chrome extension is connected, use the `mcp__claude-in-chrome__*` tools to automate browser interactions directly. Never tell the user to manually click, type, or navigate in the browser when automation is available. Take screenshots to verify UI state. If Chrome tools become disconnected, inform the user they need to restart Claude Code with `--chrome`.
- **Prefer documentation over filesystem discovery** - When needing to enumerate things (components, skills, tools, endpoints, etc.), first check for existing catalog/index documentation in the project's docs/ or CLAUDE.md. Only fall back to shell commands (glob, ls, find) if no documentation exists. If you must use filesystem discovery, that's a signal documentation is missing - create or update the catalog document after completing the immediate task.
- **Catalog documents for enumerable things** - When a project has a category of enumerable items (UI components, API endpoints, CLI commands, skills, etc.), there should be a markdown catalog document listing them all with key metadata. This catalog is the authoritative source, not the filesystem. When adding new items to such categories, update the catalog as part of the same change.
- **Maximize autonomous progress, minimize blocking on human** - When work requires human action (running commands, approvals, decisions), don't stop and wait. Instead:
  1. Add the needed action to a `HUMAN_TODO.md` file in the project root (create if missing)
  2. Continue with other available work: other threads, design docs, implementation plans, tests, documentation
  3. Only ask the human for input when ALL completable work is done
  4. The human actions file serves as a resumable queue - include enough context for each item that work can continue when the human completes it
  This keeps momentum and respects human time by batching requests rather than blocking repeatedly.
- **Ignore organizational factors** - Do not concern yourself with teams, roles, people, ownership, consensus, approvals, business risk, coordination overhead, stakeholder alignment, or any human/organizational dynamics. Focus purely on technical design and implementation. Questions like "who owns this?" or "has this been approved?" are not your job. Assume all organizational prerequisites are handled. Never flag organizational risks or blockers in critiques or plans.
- **Don't critique infrastructure scaling decisions** - When reviewing plans, don't flag "over-engineering" for infrastructure choices like Redis vs in-memory, database vs file storage, or horizontal scaling prep. These are cost/institutional decisions outside technical critique scope. Focus on whether the design works, not whether it's "too much" for MVP.
- **Semantic presumptiveness is counter-indicated for code** - When documenting, describing, or referencing code-related content:
  - Never assume a term means what you expect - verify against source
  - Never cite line numbers, page numbers, or locations without current-session verification
  - Never describe API behavior, SDK patterns, or CLI commands from memory - read the implementation
  - Never inflate counts ("150+ connectors" when source shows 99)
  - Never fabricate exit codes, version numbers, or technical contracts
  - Distinguish clearly between: (a) verified from source this session, (b) paraphrased from documentation, (c) inferred/synthesized, (d) generated/presumed
  - When in doubt, say "unverified" rather than assert confidence
  - Implementation is ground truth; documentation describes intent but may be stale
  - Source code comments are more reliable than prose documentation
  This prevents technical misinformation that could mislead developers.
- **DATA_SOURCE traceability** - Every entry in DATA_SOURCES.md must trace to either: (a) a decision made during design, or (b) code it informed. If a DATA_SOURCE was provided but not used, that is a GAP which MUST be included in any GAP_ANALYSIS. Unused data sources indicate either: missing implementation, misunderstood requirements, or scope reduction that should be documented.
- **GAP analysis includes demo requirement gaps** - When running a GAP analysis on a project that includes a DEMO, check if the demo requirements were fully satisfied. Gaps in demo requirements (missing visualizations, incomplete walkthroughs, unclear explanations) are gaps in the project itself. A demo that doesn't demonstrate all claimed functionality is incomplete.

## Testing Guidelines

**Code without tests is incomplete.** When writing implementation code, always write corresponding tests in the same session. Never say "the code is done" or "implementation complete" if tests were not written. If tests cannot be written for some reason (e.g., no test framework, unclear how to test), explicitly flag this as a gap that needs addressing. The default expectation is: code + tests together, not code now and tests maybe later.

Write tests that catch real bugs, not tests that merely exercise code paths:

- **Test at the right layer** - Unit tests should assert behavior of the unit under test, not pass through to assert something about underlying systems. If a test only passes because a dependency works correctly, it's testing the wrong thing.
- **Have a failure hypothesis** - Before writing a test, articulate what specific bug, error, or mistake it would catch. "This test would fail if X" should have a concrete X. Tests without a clear failure hypothesis are ceremony, not verification.
- **Avoid pass-through assertions** - If your test calls A which calls B which calls C, and your assertion is really about C's behavior, either:
  - Test C directly, or
  - Mock B to verify A's interaction with it
- **Test boundaries, not glue** - Focus tests on: input validation, edge cases, error handling, state transitions, integration points. Skip testing: simple delegation, trivial getters/setters, framework behavior.
- **Name tests by what breaks** - Test names should describe the failure scenario: `TestRejectsEmptyInput`, `TestHandlesTimeoutGracefully`, not `TestProcessInput` or `TestHappyPath`.
- **Bug reports require both fix and test** - When a bug or regression is reported, the fix must include a regression test that would have caught it. A bug report should always result in: (1) a fix, and (2) a test proving the fix works and preventing recurrence. Fixes without tests are incomplete.

## Reading Unstructured Notes

When processing unstructured inputs (meeting notes, brainstorms, design docs):

**Treat as untrusted:**
- Blatant assertions may be noise
- Strong phrasing doesn't equal importance
- Surface-level descriptions might miss the point

**Look for:**
- Subtle technical constraints mentioned in passing
- Brief phrases that imply larger architectural decisions
- Nuanced distinctions that reveal actual requirements
- Throwaway comments that expose hidden assumptions

**Approach:**
- Read for what's **implied**, not just stated
- Identify technical invariants buried in examples
- Distinguish between "what they want" and "what the system actually needs"
- Extract the kernel of truth from verbose descriptions
- **Unmerged branches in research repos are exploratory, not authoritative** - use their approaches but don't treat as requirements
- **Never copy literal text from unstructured notes** - always rephrase (simplify or expand) before writing to files
- **Never copy verbatim from private/proprietary context** - When given private or proprietary inputs as context (internal docs, ticket contents, Slack conversations), never copy text verbatim into files or outputs. Always rephrase. Names especially should never be propagated. If unclear whether something is private, ask.

## After Writing Complete Code

When code completely satisfies a use case (and tests would pass):
- **Always write a DEMO markdown file** showing the use case walkthrough
- Demo should be runnable by user following the steps
- Include expected output at each step
- **Especially important for user-facing features** - demos are how we validate the UX makes sense

## Reports

When running experiments, audits, or analyses that produce findings:
- **Write reports to `reports/` subdirectory** in the project root (create if missing)
- **Datestamp the filename**: `REPORT_<TOPIC>_YYYY-MM-DD.md` (e.g., `REPORT_SKILL_COMPLIANCE_2026-01-23.md`)
- **Structure**: Executive summary, methodology, findings, recommendations
- **Include raw data references**: Link to or embed the underlying data (test output, metrics, logs)
- **Reports are immutable**: Don't update old reports; create new dated versions for follow-up analysis
- **Purpose**: Reports capture point-in-time findings for historical reference and decision-making

# Requirements

- Claude MUST apply the skill defined in skills/default/dry_witted_engineering.md unless explicitly overridden.
- Claude MUST apply skills/meta/project_process.md for all projects (DATA_SOURCES.md tracking is mandatory).
- **Recursive data source investigation** - When a project references other projects or data sources, investigate those recursively if they seem relevant. Always list other projects (by full path) in your project's DATA_SOURCES.md. If a referenced project has its own DATA_SOURCES.md, review that too.
- **Fetch before learning from repos** - Always run `git fetch` (not merge) on any repo you're learning from that appears to be a git repository. This ensures you don't miss recent contributions. Do this before reading files from related projects.
- **GLOSSARY.md for terms of art** - Maintain a `GLOSSARY.md` in the project root for domain-specific terminology. When you encounter or use terms of art (e.g., "uplift", "match_baton_id", "sync"), add them with clear definitions. This prevents confusion when the same word means different things in different contexts.
- **PLAN_*.md for plan mode** - When entering plan mode, document the plan in a `PLAN_<SPECIFIC_OBJECTIVE>.md` file in the project directory (e.g., `PLAN_INVESTIGATE_MULTI_ENTITLEMENT_MATCHING.md`). This allows multiple plans to coexist. Also list all plans in order in PROJECT.md.
- For architectural decisions, apply skills/engineering/structural_constraints.md (prefer compile-time safety over runtime checks).
- For design and planning work, apply skills in skills/design/:
  - systematic_feature_design.md: Use 10-step methodology and Level Framework for new features
  - socratic_discovery.md: Use questions to build consensus when stakeholders are skeptical
  - rigorous_critique.md: Apply three-lens critique before implementation (expect 30-50% cuts)
  - complete_developer_experience.md: Ensure Tools + Documentation + Agents (all three legs)

# Permissions

- Read-only operations in cwd (ls, cat, grep, git status, etc.) do not require approval
- Full read permission on everything under /Users/rch/repo/ - no need to ask before viewing files
- When considering if a shell command should require permission to run, consider every binary invoked for each subprocess or pipe, and also consider if each command is known to be read-only.
- Build commands (`make build`, etc) should not be run directly by claude, instead, prompt the user to run the command and notify you when it is complete or has any errors
- NEVER delete files without explicit user permission. Deletion is lossy and irreversible. Always ask before removing files, even if they appear redundant or superseded.
- **Be responsible when killing processes** - Before killing a process on a port, first identify what process it is (using `lsof -i :PORT` or similar). Only kill if it's clearly a stale/orphaned process from this project. If it's an unknown process or belongs to another project, ask the user before killing.
- **Don't narc, don't snitch** - Never attribute code changes, quotations, ideas, or documents to specific people by name in files, documentation, commit messages, or any output that may be shared. Anonymize sources. Say "a colleague suggested" or "feedback indicated" rather than naming individuals. This protects identities in artifacts that may be widely distributed.
- **Project status means what remains, not what's done** - When asked for project status, check the project files (REMAINING_TODOS.md, TODO.md, backlog, etc.) and report what work is left. Don't summarize completed work - the user already knows what's done. Status = remaining work.
- **NEVER publish without explicit instruction** - Never publish, deploy, push, upload, submit, or otherwise make project content externally visible without explicit user instruction to do so. This includes: git push, npm publish, creating PRs, posting to external services, updating Notion pages, sending emails, creating GitHub issues, or any action that shares project content outside the local filesystem. Research and planning stay local until the user explicitly says to publish.
- **Branch naming convention** - User's branches follow pattern `rch/<type>/<topic>` where type indicates the nature of work:
  - `rch/feature/<thing>` - new functionality
  - `rch/bugfix/<thing>` - bug fixes
  - `rch/perf/<thing>` - performance improvements
  - `rch/logging/<thing>` - logging/observability changes
  - `rch/refactor/<thing>` - code restructuring
  - `rch/docs/<thing>` - documentation only
  - Examples: `rch/feature/cone-mcp-interactions`, `rch/bugfix/token-refresh`, `rch/perf/sync-batching`
