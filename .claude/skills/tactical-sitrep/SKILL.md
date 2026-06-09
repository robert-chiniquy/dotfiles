---
name: tactical-sitrep
description: Tactical situation report scoped to ONE proximate goal with a hard deadline. Pulls real-world signals from multiple nebulous sources (ticketing system + git host + wiki + chat + local tracker) and synthesizes a calibrated A/B/C readiness call, plus the single most calendar-pressing action. Triggers on "sitrep", "situation report", "tactical read", "am I on track for X", "where are we on <named milestone>", "deadline read", "alpha check", "milestone check", "launch check", "what's at risk before <date>".
---

# Tactical Sitrep

## Overview

A sitrep is the opposite of an orient. Orient is gestaltic — sweep everything, surface trajectory, present the world. Sitrep is proximate — pick one goal, ignore everything else, answer "are we landing this on time?" in calibrated form.

Sitreps are about HARSH SCOPE. A sitrep for an alpha milestone does not look at the downstream audit, does not look at the launch beyond it, does not enumerate the long tail. Only the items that ship-or-don't-ship at the deadline in question.

Run a sitrep when the user names a specific deadline (milestone, demo, release, audit kick-off) and wants a current read on landing it. Do not run a sitrep as a general status check; use a project orient skill for that.

## When to use

- User names a specific milestone with a known target date ("alpha", "early access", "the demo", "the launch", "the audit", "Friday's freeze").
- User asks a calibrated A/B/C question about a deadline ("am I behind / on track / ahead?").
- A deadline is approaching and the user wants to know if any in-flight work is silently failing.
- After landing a substantial commit cluster, to verify it moved the milestone needle the way expected.

## When NOT to use

- General "what's the state of the project" questions — use orient.
- "What should I work on next" with no time pressure — use a backlog sweep.
- Single-PR status check — use pr-pass or `gh pr view` directly.
- Strategic planning, scope decisions, prioritization across milestones — different skill family.

## Workflow

### Phase 1 — Pin the goal

Identify exactly one milestone, with one target date and one in-scope ticket query. Do not let the user's question drift you into broader scope. If they ask "am I on track for Alpha and EAP?" — that's two sitreps; run them separately, in series.

State explicitly: the goal name, the target date, and the number of days remaining (compute from `today`).

### Phase 2 — Enumerate scope from the authoritative tracker

Pull the in-scope items from the canonical tracker for this project. For ticketing-system-tracked work, query the milestone directly (e.g. via the ticketing system's MCP integration or CLI). For local-tracker work, the equivalent milestone-filtered list command.

Capture per item: identifier, title, status, assignee, priority, last-updated. **Do not enumerate every field; you need just enough to drive Phase 3.**

If a tracker item is marked "Duplicate", "Won't fix", "Deferred", or low-priority/unassigned with no signs of life, list it separately as "effectively descoped" and exclude from the in-scope count.

### Phase 3 — Pull real-world signals per in-scope item

This is the heart of the sitrep. The tracker tells you what state the work *claims* to be in. Real-world signals tell you whether that claim holds.

For each in-scope item, fetch (parallel-fan-out via subagents if more than ~3 items):

- **PR state**: number, draft, mergeStateStatus, mergeable, last-push timestamp, last 3 commit messages.
- **CI**: total checks + pass/fail/pending counts; first failing job name.
- **Review state**: latest review's reviewer + state; whether the PR is approved or has CHANGES_REQUESTED.
- **Comment cadence**: review-thread comments in the last 7 days. Stale threads = stalled signal.
- **Blocker hints in the PR body** ("Requires bumping X", "Blocked on Y", "Waiting on Z").
- **Chains**: does this single tracker item correspond to multiple coordinated PRs across multiple repos? If so, list each.

When chains exist, the tracker's "1 item in review" is misleading. A chain of `proto → SDK → shells → server` is four merges in a specific order; report it as such.

If the user has additional sources (wiki design docs, chat threads, internal knowledge base), fan out a third subagent to pull recent activity from those for the same set of items + assignees. Risks raised in chat but not yet ticketed are the highest-value signal a sitrep can surface; they are *the* reason to look beyond the tracker.

### Phase 4 — Cross-reference: tracker claim vs. real-world state

For each item, compare:

- Tracker says "In Review" — does the PR actually exist, is it not-draft, is CI green, are there reviewers? If draft + no reviewer, "In Review" is aspirational.
- Tracker says "Done" — is the PR actually merged, or only approved? Has it deployed to where the milestone needs it?
- Tracker says "Backlog" or low-priority — is anyone actually working on it? Recent pushes to a related branch under that author would override the tracker label.
- Headline metric ("8% complete") — does it weight the load-bearing items, or count low-priority items equally? Decompose if needed.

Surface mismatches explicitly. The user will trust the tracker less after this skill runs than before; that is the intent.

### Phase 5 — Synthesize the A/B/C call

Produce a single calibrated label:

- **A — Behind**: the deadline will be missed at current cadence, OR has a load-bearing item with no path to landing in time.
- **B — Roughly on track**: all in-scope items have plausible landing paths; the risk is in execution and coordination, not in unknown work.
- **C — Ahead**: items are landing faster than the timeline requires; buffer exists.

Do not hedge. If the answer is A, say A. If it is B, say B. If two sub-tracks are at different states (e.g. "engineering ready, decisions blocked"), call out the decisive sub-track and give that one's letter as the headline.

For each letter, give the *specific* reason in one or two bullets, citing the cross-referenced state from Phase 4.

### Phase 6 — Identify the single most calendar-pressing action

End with one action that has the highest leverage. Not three, not five. One. The criterion is: which action, if not taken in the next 48 hours, most narrows the path to landing the milestone?

Examples by shape (genericized — name the artifact, not the author):

- "Review and merge the in-review SDK PR. It is mergeable with zero reviews on it, and N downstream items are sequenced behind it."
- "Book the auditor. <N> days to kick-off-by. No PR can move this calendar item; it is a phone call."
- "Decide the gating policy-default question. One in-review server PR cannot land safely until this is resolved, and two more items are sequenced behind that."

Only follow with a fuller punch-list if the user asks for it.

## Output discipline

- Tight tables, not paragraphs.
- Cite specific PR / issue / commit IDs for every claim.
- **Every PR mentioned gets a footnote URL.** When the sitrep is written to a doc, each PR referenced (by `#number` or `repo#number`) must carry a numbered footnote linking to its full URL, collected in a `## PR references` footnote block at the bottom. The inline text stays terse (`c1#18648`); the footnote carries the clickable `https://github.com/<owner>/<repo>/pull/<number>`. One footnote per distinct PR; reuse the same marker if a PR is cited more than once. This applies to the written doc, not to a quick inline chat reply.
- One A/B/C call, headline, no hedge.
- No bold-item + list clutter.
- No effort estimates, no headcount language.
- No emojis.
- Lead with the answer (A/B/C); reasoning under it; one action at the bottom.

## Common Mistakes

- **Broadening scope under pressure.** When the picture looks complicated, the temptation is to widen scope to feel comprehensive. Don't. A sitrep for one milestone that ends up talking about the next milestone has failed.
- **Trusting "In Review" without verification.** The tracker says In Review; the PR is in draft, has no reviewers, has 0 comments in 7 days. That is not In Review in any operationally useful sense.
- **Counting items, not chains.** When one tracker item corresponds to coordinated PRs across multiple repos (proto + SDK + shells + server, for example), the work is N× the count. Don't report it as 1.
- **Confusing approved with merged.** Approved + CI green + mergeable is *one click* from merged. But while it sits unmerged, the milestone is not landed. Don't conflate.
- **Skipping the chat-side surface.** Wiki design docs, chat threads, in-flight design questions that have not been ticketed are the highest-value source for "what will surprise us." Always check.
- **Hedging the A/B/C call.** "B-leaning-A" is not a calibrated answer. Pick one. The reasoning bullets can carry nuance.
- **Listing more than one calendar-pressing action.** Three actions = no action. Pick the most-leveraged one; the user will ask for more if they want it.
- **Citing "today's progress" against a multi-week timeline.** Today's work is not a calibration anchor for a milestone; the deadline is. A great work day doesn't move you off A.
- **Forgetting the "no decision" calendar pressure.** Auditor bookings, vendor commitments, regulatory filings are calendar items that *do not* live on a PR board. Watch for them when the milestone implies one.
- **Incurious about your own subagents.** A sitrep includes itself. If a subagent you fanned out has been running materially longer than its siblings (a useful rule of thumb: > 5× the median sibling latency) and is now blocking your synthesis, report that explicitly: time elapsed, expected baseline, likely cause (MCP timeout / rate limit / search-then-fetch loop / token exhaustion), and the cost of continuing to wait. "Still running, no notification" is not a sitrep — it is the absence of one. When in doubt, ping the agent with SendMessage; if it does not respond on its next tool round, treat that as confirmation it is wedged, TaskStop it, and synthesize from the partial data with the gap stated explicitly.

## Calibration: A/B/C language

Use this language consistently when called for a calibrated read:

- **A — Behind**: "Behind on calendar-load. Reason: <specific work item or decision with no landing path>. Most pressing: <action>."
- **B — Roughly on track**: "Roughly on track. All in-scope items have plausible landing paths. Primary risk: <execution / coordination / review-velocity risk>. Most pressing: <action>."
- **C — Ahead**: "Ahead. Buffer of <N> days at current cadence. Most pressing: <protect-buffer action — usually 'don't add scope'>."

Do not use "tight but doable" as a hedge to avoid A. "Tight but doable" is B with risk noted; if "tight" actually means "will miss," it is A.

## Output template

```
# Sitrep — <milestone name> (<target date>, <N> days)

## Read: <A | B | C> — <one-line label>

## Scope

| ID | Title | Tracker status | Real-world state | Risk |
|---|---|---|---|---|
| ... | ... | ... | ... | ... |

## Cross-reference findings

- <mismatch 1: tracker vs reality>
- <mismatch 2>

## Most calendar-pressing action

<one specific action, naming the artifact and the deadline>

## What this sitrep deliberately did not look at

<list of out-of-scope items, so the user knows what was excluded>

## PR references

[^1]: https://github.com/<owner>/<repo>/pull/<number>
[^2]: https://github.com/<owner>/<repo>/pull/<number>
```

## Iterating this skill

When a sitrep surfaces a class of mismatch not yet listed in [Common Mistakes](#common-mistakes), add it. Examples worth watching for:

- A tracker item that decomposes into N coordinated artifacts (chain pattern).
- A "started" item that has not been pushed to in >5 days.
- A milestone where the in-scope item count does not match the rolled-up tracker count (definition mismatch).
- A calendar-pressing item that lives in a calendar tool, not a code tracker.

Each addition makes future sitreps more calibrated by default.
