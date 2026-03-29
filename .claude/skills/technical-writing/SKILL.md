---
name: technical-writing
description: |
  Long-form technical writing voice for blog posts, deep dives, architecture
  explanations, and talks. Use when writing articles, READMEs with narrative,
  or any content for external audiences beyond the team.
---

# Technical Writing

For content that will be read by people outside the immediate project.

## Voice

* Clear, direct, concrete
* Show the reader something — a code snippet, a diagram, a before/after
* Explain the problem before the solution
* One idea per paragraph
* Use headers as a scannable outline

## Structure

1. Open with what the reader will learn and why it matters
2. Set up the problem with a concrete example
3. Walk through the solution showing real code
4. Address edge cases and limitations honestly
5. Close with what to do next

## Techniques

* **Concrete over abstract.** "The query takes 400ms because it scans 2M rows" not "Performance can be problematic at scale."
* **Show, don't label.** Demonstrate the technique working. Don't say "this elegant approach."
* **Foreshadow complexity.** When simplifying, say so: "We'll ignore X for now. It matters when Y."
* **Use code as proof.** Every claim about behavior should have a snippet that demonstrates it.

## Common Mistakes

* Starting with history ("In the beginning, there was...") instead of the point
* Hedging every statement ("It could potentially perhaps help to...")
* Explaining what something IS without showing what it DOES
* Writing for peer review instead of for learning
