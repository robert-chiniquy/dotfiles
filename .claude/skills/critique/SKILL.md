---
name: critique
disable-model-invocation: true
description: |
  Manual alias for pipeline stage 2 critique (same job as rigorous-critique).
  Invoked via /critique after PLAN_*.md. See project-process.
---

# Critique

**Same job as `rigorous-critique` (stage 2).** Prefer loading that skill; this
file is the `/critique` entry. Apply the lenses to the **plan** (and design if
present), not as a free-floating essay.

## Before finishing

- [ ] Findings numbered with lens + fix?
- [ ] Plan file updated or rejected?

## Lenses

## Lens 1: Unnecessary Complexity

* What could be removed without losing the core value?
* Which components exist for hypothetical future needs?
* Where are abstractions hiding simple operations?
* What would the simplest version look like?

## Lens 2: Missing Fundamentals

* What happens on the first error?
* What happens at 10x the expected load?
* How does this fail when a dependency is down?
* What's the recovery path from data corruption?
* Where are the implicit assumptions?

## Lens 3: Feasibility Gaps

* Which components have we never built before?
* Where are we estimating based on hope rather than measurement?
* What are the integration boundaries we haven't tested?
* Which third-party dependencies are we trusting blindly?

## Lens 4: Scope Mismatch

* Does the implementation match what was actually asked for?
* Are we building Level 2 (polish) before Level 0 (platform)?
* What's the gap between the demo and production?
* Which features are solving the wrong problem?

## Lens 5: Overcorrection

Apply `$overcorrection-review` to the design and to the fixes proposed by the
other lenses.

* Which proposed cuts or guardrails rely on unmeasured difficulty, cost, risk,
  or value?
* Has a staged choice become a permanent exclusion?
* Does each mitigation match the failure it addresses?
* What evidence would justify the stricter choice?

## Output Format

For each problem found:
1. Which lens caught it
2. The specific problem (one sentence)
3. The risk if unaddressed
4. A concrete fix (not "think about it more")

Number problems sequentially. The list IS the deliverable.
