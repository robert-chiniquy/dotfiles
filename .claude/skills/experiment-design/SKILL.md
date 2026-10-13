---
name: experiment-design
description: |
  Design and run an experiment in any domain as a bounded search: the
  problem gives an objective (a number, a pass rate, or a pairwise
  preference), a threshold, and the smallest difference worth acting on;
  the solution gives factors to vary. Rounds are handed to whoever runs
  them (a scientist, a student, an ad platform, a command) and read back
  until a stated verdict. Use when asked to design an experiment, plan an
  A/B test, tune a configuration by trial, or find out whether a goal is
  reachable with the knobs available.
---

# Experiment design

Design source and citations: `/Users/rch/repo/research/shell/experiment-design/`
(`DESIGN_EXPERIMENT_DESIGN_SKILL.md`, `README.md` `## Sources`). File layouts
are in `references/formats.md`, comparison rules in
`references/comparison-rules.md`, worked examples in `references/examples.md`.

## Common Mistakes

1. Starting without δ, the smallest difference the user would act on. Every
   verdict is stated relative to δ. Ask "what is the smallest change that
   would make you switch?" δ has no default.
2. Comparing runs across rounds. Drift between days, trays, or traffic
   weeks reads as an effect. The incumbent is re-measured in every round and
   comparisons use only that round's runs, paired by block.
3. Calling a few non-improving tries a local optimum. LOCAL_OPTIMUM and
   PLATEAU are test results over every reasonable neighbor plus a frontier
   sample of size n (below).
4. Correcting the move decision for the number of neighbors. A wrong move
   costs one round. Error control belongs to the confirmation run and the
   verdict tests.
5. Reading A/B results before the planned sample size. Fix n per round and
   read once.
6. Changing δ, α, threshold, or factors after results arrive without
   starting a new protocol version.
7. One-factor-at-a-time screening. Use a main-effects design.
8. Dropping a run because its note looks bad without saying so in the
   decision log.

## Inputs (phase 0)

Help the user write `PROTOCOL.md` (layout in `references/formats.md`).
MUST have before any run:

- Objective kind and direction; threshold (absolute, or a margin over the
  baseline); δ (a win-rate margin for preferences); α (default 0.05).
- Measurement procedure, blinding if possible, guardrails, budget, largest
  batch that can run at once, blocks (days, benches, traffic slices).
- Factors: kind (ordered, unordered, integer, continuous), allowed values,
  finest step, hard to change. Many-level unordered factors get attributes
  from a named standard; each attribute value MUST appear in at least two
  levels.
- Infeasible combinations, baseline, neighbor rule.
- Reasonable moves: the neighbor moves the domain finds plausible. Propose
  them; the user approves.
- Coverage q for the rest of the frontier (default 0.2). Sample size n is the
  smallest n with (1 − q)^n ≤ α: 14 for q = 0.2, 29 for q = 0.1, at α = 0.05.

Two configurations and nothing else to vary: run phase 1 and one comparison.
Budget below phase 1 needs: report the smallest δ the budget can detect.

## Phases

1. Noise: run the baseline about 4 times across blocks. Size replicates per
   arm for the not-better-by-δ test at α, 80% power.
2. Screening, 4 or more factors (or attributes): Plackett-Burman, or a
   definitive screening design when curvature is likely. Fix factors with no
   effect above δ at their cheapest level.
3. Starts: baseline first, then greedy farthest-point picks over feasible
   configurations (distance in `references/formats.md`). Skip a candidate
   when a measured configuration near it is already better.
4. Local search per start. Step per ordered factor starts at a quarter of its
   span. Each round: incumbent plus every reasonable neighbor, replicated,
   randomized, incumbent in every block, written as `ROUND_<n>.md`.
   - Some neighbor beats the incumbent by δ at α: the best of those becomes
     the incumbent.
   - None: halve steps above their finest step and recompute replicates.
   - At finest steps with no move: one round on a random sample of n
     neighbors from outside the reasonable moves (all of them if n or fewer),
     seed recorded. A sampled neighbor that beats the incumbent by δ becomes
     the incumbent; otherwise apply the verdict tests.
5. Confirmation: a configuration at the threshold reruns in a fresh block
   with the baseline. One-sided test against the threshold at α.

## Verdicts

Each tested neighbor gets two one-sided tests at α: not better by δ, and
worse. A claim about all tested neighbors holds when each test rejects at α
(intersection-union test; no multiplicity correction).

- SOLVED: confirmation passed.
- LOCAL_OPTIMUM: finest steps; every tested neighbor shown worse.
- PLATEAU: finest steps; every tested neighbor shown not better by δ, some
  not shown worse. List those; they may be chosen on cost.
- BUDGET: budget spent first. List unresolved neighbors.
- GUARDRAIL_HALT, DUPLICATE (reached a basin an earlier start found).

Whole search: SOLVED if any start is. UNSAT when every non-duplicate start is
LOCAL_OPTIMUM or PLATEAU and the last 3 starts all landed in known basins (or
budget cannot finish another start). INCONCLUSIVE when any start is BUDGET.
Report δ, α, q, rules used, p-values, seeds, and the factors sitting at the
edge of their allowed values (candidates to widen; that is the user's call).

State the two claims separately: exact at α for reasonable neighbors; for
the rest, a sampling bound (fewer than q of them better by δ, at confidence
1 − α). Neither covers configurations more than one move away.

## Before finishing

- [ ] δ, α, q, reasonable moves recorded in `PROTOCOL.md` before phase 1.
- [ ] Every round's comparisons used only that round's runs.
- [ ] Every verdict names its tests, p-values, and seeds.
- [ ] Notes on runs addressed in `DECISIONS.md`.
