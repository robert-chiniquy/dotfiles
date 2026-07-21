---
name: tactical-sitrep
description: Tactical situation report scoped to ONE proximate goal with a hard deadline. Pulls real-world signals from multiple nebulous sources (ticketing system + git host + wiki + chat + local tracker) and synthesizes a calibrated A/B/C readiness call, plus the single most calendar-pressing action. Triggers on "sitrep", "situation report", "tactical read", "am I on track for X", "where are we on <named milestone>", "deadline read", "alpha check", "milestone check", "launch check", "what's at risk before <date>".
---

# Tactical Sitrep

One milestone, one target date, one calibrated A/B/C answer to "are we landing this on time?" Harsh scope: only items that ship-or-don't-ship at that deadline — not the next milestone, not the long tail. Two milestones = two sitreps, run in series. Also useful after landing a substantial commit cluster, to verify it moved the milestone the way expected.

Not this skill: general project state (orient), "what next" with no time pressure (backlog sweep), single-PR status (pr-pass / `gh pr view`), prioritization across milestones.

## Workflow

1. **Pin the goal.** State goal name, target date, days remaining (compute from today).

2. **Enumerate scope** from the authoritative tracker via a milestone-filtered query. Per item: id, title, status, assignee, priority, last-updated. Items marked Duplicate / Won't fix / Deferred, or unassigned low-priority with no signs of life: list separately as "effectively descoped" and exclude from the in-scope count.

3. **Pull real-world signals per item** (parallel subagent fan-out if more than ~3 items). The tracker says what state the work *claims* to be in; these say whether the claim holds:
   - PR state: number, draft, mergeStateStatus, mergeable, last-push timestamp, last 3 commit messages.
   - CI: pass/fail/pending check counts; first failing job name.
   - Review: latest reviewer + state; approved vs CHANGES_REQUESTED.
   - Comment cadence: review-thread comments in the last 7 days (silent = stalled signal).
   - Blocker hints in the PR body ("Blocked on Y", "Requires bumping X").
   - Chains: one tracker item spanning coordinated PRs across repos. `proto → SDK → shells → server` is four ordered merges — report it as such, never as 1 item.
   - If the user has wiki / chat / knowledge-base sources, fan out another subagent for recent activity on the same items + assignees. Risks raised in chat but never ticketed are the highest-value signal a sitrep surfaces.

4. **Cross-reference tracker claim vs reality**; surface every mismatch:
   - "In Review": does the PR exist, non-draft, CI green, reviewer assigned? Draft + no reviewer = aspirational.
   - "Done": merged, or only approved? Approved + green + mergeable is one click from merged, but unmerged is not landed. Deployed where the milestone needs it?
   - "Backlog" / low-priority: recent pushes to a related branch by that author override the label.
   - Headline metric ("8% complete"): does it weight the must-ship items, or count low-priority items equally? Decompose if needed.

5. **A/B/C call.** One letter, no hedge; nuance goes in the reasoning bullets (1-2, citing step-4 state). If sub-tracks diverge (engineering ready, decisions blocked), the decisive sub-track's letter is the headline.
   - **A — Behind**: will miss at current cadence, OR a must-ship item has no path to landing in time. Language: "Behind on calendar-load. Reason: <item or decision with no landing path>. Most pressing: <action>."
   - **B — Roughly on track**: every in-scope item has a plausible landing path; residual risk is execution/coordination, not unknown work. Language: "Roughly on track. All in-scope items have plausible landing paths. Primary risk: <execution / coordination / review-velocity risk>. Most pressing: <action>."
   - **C — Ahead**: landing faster than the timeline requires; buffer exists. Language: "Ahead. Buffer of <N> days at current cadence. Most pressing: <protect-buffer action — usually 'don't add scope'>."

   "B-leaning-A" is not calibrated; pick one. "Tight but doable" is B with risk noted — if tight means will-miss, it is A.

6. **Single most calendar-pressing action.** Exactly one: the action that, if not taken in the next 48 hours, most narrows the path to landing. Name the artifact, not the author. It may not be a PR at all — auditor bookings, vendor commitments, regulatory filings live on no PR board; watch for them whenever the milestone implies one. Fuller punch-list only on request.

## Output

- Lead with the letter; reasoning under it; the one action at the bottom.
- Tight tables, not paragraphs. Cite specific PR / issue / commit IDs for every claim.
- **Footnote rule** (written docs, not quick inline chat replies): every PR cited gets a numbered footnote with its full `https://github.com/<owner>/<repo>/pull/<number>` URL, collected in a `## PR references` block at the bottom. Inline text stays terse (`c1#18648`). One footnote per distinct PR; reuse the marker on repeat citations.

```
# Sitrep — <milestone name> (<target date>, <N> days)

## Read: <A | B | C> — <one-line label>

## Scope

| ID | Title | Tracker status | Real-world state | Risk |
|---|---|---|---|---|

## Cross-reference findings

- <mismatch: tracker vs reality>

## Most calendar-pressing action

<one specific action, naming the artifact and the deadline>

## What this sitrep deliberately did not look at

<out-of-scope items, so the user knows what was excluded>

## PR references

[^1]: https://github.com/<owner>/<repo>/pull/<number>
```

## Common Mistakes

- **Anchoring on today's progress.** A great work day is not a calibration anchor against a multi-week timeline; the deadline is. It doesn't move you off A.
- **Incurious about your own subagents.** A sitrep includes itself. If a fanned-out subagent runs > ~5× the median sibling latency and blocks synthesis, report it: elapsed time, baseline, likely cause (MCP timeout / rate limit / search-then-fetch loop / token exhaustion), cost of waiting. Ping it via SendMessage; no response on its next tool round = wedged — TaskStop it and synthesize from partial data with the gap stated explicitly.

When a sitrep surfaces a mismatch class not yet listed here, add it.
