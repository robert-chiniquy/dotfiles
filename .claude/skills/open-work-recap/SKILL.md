---
name: open-work-recap
description: >
  Always active in any coding context. At every stopping point that is NOT a
  direct answer to a discrete user question, default to appending a recap:
  first, a list — with full URLs — of every OPEN PR / ticket / issue that
  still needs action or has a live status (NEVER merged / closed / done items),
  then a short "Next:" section of the concrete next work the agent will do.
  Triggers on: end of a work turn, stopping point, handing control back,
  "where do things stand", status recap, open PRs, open tickets, what's next.
---

# Open-work recap at every stopping point

In any coding context, when a turn ends in **work or a status report** — i.e. at
any stopping point that is NOT simply answering a discrete user question — the
DEFAULT is to close the turn with a two-part recap, in this order.

**Empty lists belong nowhere.** Never print an empty section, a zero-item list,
or a placeholder line ("none", "N/A", "No open PRs"). A section appears ONLY
when it has at least one real item; otherwise omit it entirely. If BOTH sections
would be empty, omit the whole recap.

## 1. Open items (with URLs)

List every relevant **open** PR, ticket, or issue that still needs an action or
has a live status worth reporting. One per line, each with its **full URL
inline** (never a bare `#123` / `IGA-1234` — hyperlink it):

```
- [<id>](full-url) — <status> — <what it needs / what's blocking> — deps: <ids>
```

- **Open and actionable only.** NEVER list merged, closed, or DONE/resolved
  items — at a stopping point they are noise that buries the live ones. If an
  item's only remaining status is DONE / CLOSED / MERGED, drop it.
- **Show deps when they fit.** If the line has room, append the item's
  dependencies — what it's blocked by, waiting on, or built atop (hyperlink any
  that are themselves PRs/tickets). Deps make the action/merge order obvious at
  a glance. Omit the segment when there are none, or when the line is already
  long enough that deps would hurt readability.
- Include: PRs awaiting review / merge / CI, failing checks, unresolved review
  threads, tickets in progress or blocked, issues waiting on a decision — any
  live loop.
- **Nothing open? OMIT the entire section** — no empty list, no "none" line.
  Show it only when it has at least one real item (per "Empty lists belong
  nowhere" above).

## 2. Next

Then, under the short heading **`Next:`**, the concrete next work the agent will
do. **Number every step** (`1.`, `2.`, `3.` …) so the user can refer to one by
number ("do 2"). Keep the numbering **stable across recaps**: a live step keeps
its number turn-to-turn; when a step is done, drop it but do NOT renumber the
survivors (leave the gap), and give a genuinely new step the next unused number
— so "#3" always means the same thing. One terse line each, not prose. No next
work? **Omit this section** — no placeholder.

## When NOT to recap

- The turn is a **discrete user question** (a lookup, yes/no, or explanation) —
  just answer it; no recap.
- Both sections would be empty (nothing open, nothing next) — a recap is noise;
  skip it.

## Why

At a stopping point the user's first question is always "what's live, and what's
next?" Surfacing the open loops (with clickable URLs) and the next action every
time means they never have to ask or scroll back. Merged / closed items are
finished — listing them only hides the ones that still need attention.
