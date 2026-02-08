# Commit & PR Style Guide

## Commit Messages

**Title line:**
- Lowercase start (unless proper noun, ticket ref, or acronym)
- Imperative or descriptive — both fine
- Short: aim for under 72 chars before the PR number suffix
- Ticket refs in brackets prefix when relevant: `[DX-122]`
- No trailing period

**Body (optional, for non-trivial changes):**
- Blank line after title
- Explain *what* and *why*, not *how* (the diff shows how)
- Use `- ` bullets for multi-part changes
- Can be terse — single sentence is fine
- Sub-commits in multi-commit PRs are informal (`* fmt`, `* lint fix`)

**Factoring:**
- Single logical change per commit in simple PRs
- Multi-commit PRs common for: fix + followup lint/fmt, or multi-part features
- Squash-merge is the norm (hosting platform adds PR number suffix)
- Don't over-split — a proto change + handler change + generated code = one commit

## PR Titles

- Same style as commit titles (since squash-merge uses the PR title)
- Under 72 chars when possible
- Ticket refs in brackets prefix: `[DX-122]`
- Descriptive of the *outcome*, not the *activity*

## PR Descriptions

**Minimal PRs** (1-3 files, obvious change): body can be empty or one line.

**Substantial PRs** use this loose structure:
```
## Summary
1-3 sentences or bullets explaining what and why.

## What's in this PR (optional, for multi-part changes)
**Area 1:** description
**Area 2:** description

## Reviewer notes (optional)
- Non-obvious decisions, things that look wrong but aren't, gotchas

## Test plan (optional)
- [ ] Checklist items
```

**Patterns:**
- Bold section headers with `**text**` or `##` — both used
- Bullets with `- ` for lists
- Code references use backticks: `FunctionName()`, `field_name`
- No boilerplate — skip sections that don't apply
- Screenshots included for UI changes
- "Depends on #NNNN" for dependency chains
