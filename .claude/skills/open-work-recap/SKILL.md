---
name: open-work-recap
description: >
  End-of-coding-turn recap checklist. Load when a coding turn stops (not a
  discrete Q&A answer): open PRs/tickets with full URLs, then numbered Next
  steps. Triggers: stopping point, handoff, "where do things stand", open PRs,
  what's next. Not always-on — apply at turn end only.
---

# Open-work recap (end of coding turn)

**When:** coding/status turn ends. **Skip:** pure Q&A; or both sections empty.

## MUST

1. List only **open/actionable** PRs/tickets/issues (never merged/closed/DONE).
2. Each PR list line: **bare full URL as visible text**, then title  
   `1. https://github.com/org/repo/pull/123 Title`  
   (Claude.md: multi-PR lists use bare URLs, not markdown anchors.)
3. Number Next steps uniquely and stably across turns (do not renumber survivors).
4. **Omit empty sections** — no "none", no empty headings.

## Checklist (before send)

- [ ] Any live open item missing?
- [ ] Every multi-PR line has a copyable full URL?
- [ ] Next steps numbered and concrete?
- [ ] Empty sections removed?

## Line shapes

```
1. https://github.com/org/repo/pull/123 Title of PR

Next:
1. rebase #123 onto main
2. address remaining thread on auth
```

Deps/status prose after the list is fine; do not bury URLs in `[text](url)` inside multi-PR lists.
