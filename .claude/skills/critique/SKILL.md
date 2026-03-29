---
name: critique
disable-model-invocation: true
description: |
  Systematically find problems in a design before building it. Four lenses:
  unnecessary complexity, missing fundamentals, feasibility gaps, and scope
  mismatch. Use after creating an implementation plan or when a design feels
  wrong. Invoked via /critique.
---

# Critique

Apply four lenses to the design. Each lens asks different questions.

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

## Output Format

For each problem found:
1. Which lens caught it
2. The specific problem (one sentence)
3. The risk if unaddressed
4. A concrete fix (not "think about it more")

Number problems sequentially. The list IS the deliverable.
