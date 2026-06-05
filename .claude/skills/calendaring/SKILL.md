---
name: calendaring
description: |
  Build and iterate on a personal master schedule across travel, parenting,
  school/camp, festivals, conferences, administrative deadlines, and work
  travel. Pulls from Google Calendar + web research. Emits a single
  date-prefixed line per item, chronologically ordered, with [Owner] tags
  and explicit conflict flags. Use when the user is organizing dates over
  a multi-month window, planning around camps and trips, reconciling work
  travel against family commitments, or preparing a shareable schedule.
  Triggers on: organize dates, my calendar, schedule, plan june (any
  month), what's on, family schedule, camp schedule, conference dates,
  reconcile conflicts.
---

# Calendaring

Single-purpose: produce a skimmable, chronologically-ordered master list
of dated items spanning the requested window, drawn from the user's
Google Calendar plus targeted web research, plus items the user feeds in
during iteration.

## Output format — strict

Each item is exactly one line. No multi-line bullets, no sub-bullets,
no inline tables. Each line begins with a date or date range, followed
by an `[Owner]` tag (when relevant), then the description.

```
DEFAULT — Work-from-home: M/T; <city> office: W/Th/F (<user>)
RECURRING <weekday> <time> — [Owner] description
M/D <Dow> — [Owner] description
M/D <Dow> H:MM AM/PM — [Owner] description
M/D–M/D — [Owner] description (multi-day item)
TBD — [Owner] description (no date assigned yet)
```

Owner tags use brackets and short names — first name of the person or
people involved. For shared/admin items where ownership is obvious
(holidays, public events) the tag may be omitted or set to `[Holiday]`.
Common patterns: `[<self>]`, `[<child>]`, `[<self>+<child>]`,
`[<co-parent>+<child>]`, `[Holiday]`.

Status modifiers (in the description, not as separate items):
- `(MAYBE)` for tentative trips/events
- `(NEEDS RESPONSE ...)` for pending action items embedded in events
- `(DEADLINE: ...)` for pay/file/cancel-by dates
- `(conflicts X)` to flag explicit overlap with another listed item

## Sources to pull from

1. **Google Calendar MCP tools** — list_events on both work and personal
   calendars over the requested window. Use eventTypeFilter for
   `workingLocation` to extract the home/office pattern. Filter out
   recurring routine items (trash bins, daily standups) unless asked.
2. **Web search** for known public dates:
   - Conferences (fwd:cloudsec, Black Hat, DEF CON, etc.)
   - Festivals (multi-day music festivals, etc.)
   - Election days (state and federal)
   - Tax deadlines (estimated quarterly, fiscal year-end)
   - Property tax due dates (county-specific)
   - Holidays
3. **User-provided items** during iteration — add verbatim, place in
   chronological order, don't paraphrase the user's wording unless
   asked.

## Filter rules

**Include:**
- Travel days (departures, returns, in-transit)
- Family/child-related camps, school, classes (one line per range or
  one per occurrence if user requests)
- Conferences and one-day events
- Tax/legal/financial deadlines
- Holidays
- Non-recurring work events (interviews, onsites, parties)
- Specific work meetings the user names

**Exclude (unless user asks):**
- Recurring work meetings (standups, weeklies, brown bags, 1:1s)
- Routine recurring chores (trash bins, etc.)
- Declined events from other people's calendars

## Recurring kid activities

Two acceptable forms; let the user pick:
- **Series form**: one `RECURRING <Dow> <time> — [<child>] <activity>`
  line at the top of the list.
- **Per-occurrence form**: one line per actual occurrence with the
  specific date. Use this when the series has a finite end and the user
  wants to see conflict overlap with travel/camps.

## Conflict detection

When two items overlap a date or time:
- Add `(conflicts X)` or `(potential conflict with X)` inside the
  description of one or both lines.
- Surface `TBD — <description>` items for resolution actions (e.g.,
  `TBD — [<self>] child pickup coverage Aug 3–7 — out of town`).

## What NOT to put inside the artifact

The artifact is a standalone document. Do not include:
- Meta-commentary about the iteration process
- References to "the window," "this list," "next steps," "what's next"
- The phrase "I'll keep accumulating" or anything narrating the workflow
- Sources/citations (those go in the chat reply, outside the code block)

Anything procedural goes in the conversational reply *around* the
artifact, not inside it.

## Iteration protocol

The user feeds new items one at a time. Each turn:

1. Re-print the **entire** updated list (do not show only the diff).
2. Insert new items in chronological order.
3. Apply any structural change requested (e.g., "remove the Sat
   capoeira line", "label items which are mine with my name").
4. Keep the artifact self-contained per the rule above.
5. After the artifact, optionally ask a single clarifying question or
   flag a single new conflict. One bit of post-artifact text max.

If the user asks for blocking clarification before producing the list,
ask. Otherwise produce the list and ask the question after.

## Publishable variant

When the user says they want to publish/share the list, strip
identifying details from sensitive lines:
- Replace specific event names with generic ones (named festival
  → "<state> trip", named wedding → just "wedding")
- Remove names of friends/family they didn't pre-approve for sharing
- Keep dates, durations, and ownership tags

Maintain the same single-line, chronological format.

## Google Calendar write actions

Calendar writes require explicit user authorization per their standing
rule. When a calendar tool would help (create event, set
workingLocation pill), state the limitation and ask before calling. The
default action is to *show* the updated list, not modify Google
Calendar.

`mcp__claude_ai_Google_Calendar__create_event` does **not** expose
`eventType: "workingLocation"` — those special pills must be set by
the user in the Calendar UI. Don't fake one with a normal all-day
event.

## Defaults to surface early

When starting a new window, pull the workingLocation recurring pattern
from the calendar and surface it as the first line:

```
DEFAULT — Work-from-home: <days>; <city> office: <days> (<user>)
```

Note explicit deviations (camping departure days, conference travel)
as separate items that displace the default for those specific dates.

## Common admin items to consider per window

- Federal/CA estimated tax due dates (Q1 4/15, Q2 6/15, Q3 9/15, Q4 1/15)
- CA fiscal year-end (6/30)
- Property tax installment dates (county-specific)
- Vehicle registration (rolling per vehicle — ask for date)
- Driver's license / passport expirations (ask)
- Election days (state primary, general)
- Camp deposit/final-payment deadlines (ask)

Surface a `TBD — [Owner] <verify/check item>` line for any of these
where the user hasn't given a specific date.
