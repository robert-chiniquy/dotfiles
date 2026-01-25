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

## Self-Check

Before any phase complete:
- [ ] DATA_SOURCES.md current
- [ ] LEARNINGS.md captures discoveries
- [ ] HUMAN_ACTIONS_NEEDED.md empty or truly blocked
- [ ] Deprecated code in old/ with docs
- [ ] Sensitive docs in private/
- [ ] DEMO.md for user-facing features
- [ ] Documents versioned, not overwritten
- [ ] Checkpoint commit created
- [ ] Catalog documents updated
