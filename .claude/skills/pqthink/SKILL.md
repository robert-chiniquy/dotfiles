---
name: pqthink
disable-model-invocation: true
description: |
  Pragmatic architecture judgment. Six-pass analysis for design decisions:
  what works, what breaks, what ships, what costs, what simplifies, what
  stays. Use when choosing between approaches or evaluating whether to
  build something. Invoked via /pqthink.
---

# PQThink

Pragmatic CTO-level judgment. Six passes, each asking one question.

## Pass 1: Does it work?

Trace the execution path end to end. Identify every assumption.
What data flows where? What are the error paths? Does the happy path
actually produce the correct result?

If you can't trace it, it doesn't work.

## Pass 2: How does it break?

Failure modes, not features. What happens when:
* The database is slow (10x normal latency)
* A dependency returns garbage
* Two requests hit the same resource simultaneously
* The input is 1000x larger than expected
* The network drops mid-operation

For each: is the failure graceful, noisy, or silent?

## Pass 3: Does it ship?

Can this be deployed incrementally? What's the migration path?
Is there a rollback plan? How do you know it's working in production?

If deploying requires a flag day, rethink the approach.

## Pass 4: What does it cost?

Not money — maintenance. Who understands this code in 6 months?
How many moving parts? How many config knobs? How many external
dependencies? Each one is a future incident.

The cheapest code is code that doesn't exist.

## Pass 5: What simplifies?

What could be deleted? What's solving a problem that doesn't exist yet?
Where is complexity serving the design vs serving anxiety about the future?

Find the version that's half the size and delivers 80% of the value.

## Pass 6: What stays?

Which decisions are reversible and which are permanent?
Reversible decisions should be made fast. Permanent decisions
(data formats, public APIs, security boundaries) deserve this analysis.
Everything else: pick one and move on.

## Output

One paragraph per pass. End with a recommendation: build it, cut it,
simplify it, or defer it. The recommendation must be specific enough
to act on without further discussion.
