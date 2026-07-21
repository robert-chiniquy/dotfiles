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

## Output format — strict

One line per item, chronological order. Date or range first, then `[Owner]`
tag (when relevant), then description. No multi-line bullets, sub-bullets,
or tables.

```
DEFAULT — Work-from-home: M/T; <city> office: W/Th/F (<user>)
RECURRING <weekday> <time> — [Owner] description
M/D <Dow> — [Owner] description
M/D <Dow> H:MM AM/PM — [Owner] description
M/D–M/D — [Owner] description (multi-day item)
TBD — [Owner] description (no date assigned yet)
```

Owner tags are bracketed first names: `[<self>]`, `[<child>]`,
`[<self>+<child>]`, `[<co-parent>+<child>]`, `[Holiday]`. Omit for obvious
shared/public items.

Status modifiers inside the description: `(MAYBE)`, `(NEEDS RESPONSE ...)`,
`(DEADLINE: ...)`, `(conflicts X)`.

## Sources

1. **Google Calendar MCP** — list_events on both work and personal
   calendars. Use eventTypeFilter `workingLocation` for the home/office
   pattern. Skip recurring routine items (trash bins, standups) unless
   asked.
2. **Web search** for public dates: conferences (fwd:cloudsec, Black Hat,
   DEF CON, etc.), festivals, election days, tax deadlines, county
   property tax dates, holidays.
3. **User-provided items** — insert verbatim in chronological order; don't
   paraphrase.

## Filters

Include: travel days, camps/school/classes, conferences, tax/legal/financial
deadlines, holidays, non-recurring work events, work meetings the user names.

Exclude unless asked: recurring work meetings (standups, weeklies, 1:1s),
routine chores, declined events.

## Recurring kid activities

Either one `RECURRING <Dow> <time> — [<child>] <activity>` line at top, or
one line per occurrence — use per-occurrence when the series has a finite
end and overlap with travel/camps matters. Let the user pick.

## Conflicts

Flag overlaps with `(conflicts X)` or `(potential conflict with X)` in the
description. Add `TBD — [Owner] <resolution action>` lines (e.g.,
`TBD — [<self>] child pickup coverage Aug 3–7 — out of town`).

## Artifact hygiene

The artifact is standalone: no meta-commentary, no workflow narration, no
"next steps", no sources/citations inside it. Procedural text goes in the
chat reply around it.

## Iteration

Each turn: re-print the ENTIRE updated list (never a diff), new items in
chronological order. After the artifact, at most one clarifying question or
one new conflict flag.

## Publishable variant

When asked to publish/share: replace sensitive event names with generic
ones (named festival → "<state> trip", named wedding → "wedding"), drop
names of people not pre-approved for sharing, keep dates/durations/owner
tags and the same format.

## Workflowy storage (if syncing)

Master schedule = 📆 Calendar node with month-folder children. Invariants:

1. No active item marked completed — completed nodes are hidden in the
   Workflowy UI. Any month with current/future dates must be uncompleted.
   When the user reports "I can't see X", check completion state first.
2. Month folders chronological across years, with year-suffixed labels
   (`Aug 2026`, not bare `Aug`).
3. Day-tagged items (`<time startDay="N">`) sorted ascending within their
   month; re-sort after batch inserts. Untagged items may sit top or
   bottom.

## Google Calendar writes

Default is to show the updated list, not modify the calendar; ask before
any write. `create_event` does NOT expose `eventType: "workingLocation"` —
those pills are set by the user in the Calendar UI. Don't fake one with a
normal all-day event.

## Defaults to surface early

Pull the workingLocation pattern first and emit it as the leading
`DEFAULT —` line. Note deviations (camping departures, conference travel)
as separate items displacing the default on those dates.

## Admin items per window

- Federal/CA estimated taxes: Q1 4/15, Q2 6/15, Q3 9/15, Q4 1/15
- CA fiscal year-end 6/30
- Property tax installments (county-specific)
- Vehicle registration (rolling per vehicle — ask)
- License/passport expirations (ask)
- Election days (state primary, general)
- Camp deposit/final-payment deadlines (ask)

Emit `TBD — [Owner] <verify/check item>` for any without a user-given date.
