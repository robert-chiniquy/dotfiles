---
name: technical-writing
description: |
  Long-form technical writing voice AND structure for content read outside the
  team: blog posts, deep dives, architecture explanations, conference talks,
  narrative READMEs. Use when drafting, rewriting, or polishing an external
  technical article. Distinct from dry-engineering (terse work comms) and
  casual-slack-tone (chat).
---

# Technical Writing

For content read by people outside the immediate project. The explaining voice:
broader audience, deeper subject, the goal is to bring people along.

## Voice

* The voice of a learner who builds: reads the textbook on the weekend, still a
  little amazed the thing works, wants you to see why.
* Relentlessly optimistic about the domain; honest about scope and limits.
* Clear, direct, concrete. One idea per paragraph.
* Show the reader something: a code snippet, a diagram, a before/after.
* Explain the problem before the solution.
* Headers are a scannable outline.
* Never academic, never marketing, never tutorial, never lecture, never sententious.

## Structure

1. Open with what the reader will learn and why it matters
2. Set up the problem with a concrete example
3. Walk through the solution showing real code
4. Address edge cases and limitations honestly
5. Close with what to do next

## Techniques

* **Concrete over abstract.** "The query takes 400ms because it scans 2M rows,"
  not "Performance can be problematic at scale."
* **Show, don't label.** Demonstrate the technique working. Don't call it elegant.
* **Discovery framing.** "It turns out that..." carries the reader through a
  finding instead of announcing the conclusion first.
* **Understatement for emphasis.** Let a large result land in a plain sentence.
* **Foreshadow complexity.** When simplifying, say so: "We'll ignore X for now.
  It matters when Y."
* **Code as proof.** Every claim about behavior has a snippet that shows it.

## Examples

Abstract claim vs. concrete proof:
* Before: "Retries can amplify load during an outage."
* After: "Each client retried three times, so the failing service saw 4x its
  normal traffic at the worst possible moment."

Announced conclusion vs. discovery framing:
* Before: "The cache key was the bug."
* After: "It turns out two requests that looked identical hashed to the same
  cache key, and only one of them was allowed to read the result back."

## Common Mistakes

* Starting with history ("In the beginning...") instead of the point
* Hedging every statement ("It could potentially perhaps help to...")
* Explaining what something IS without showing what it DOES
* Writing for peer review instead of for learning
