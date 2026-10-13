# Examples

## Science fair, worked by hand

All numbers in this section are synthetic, made up to show the procedure.

Protocol: the `PROTOCOL.md` example in `formats.md`. Paper airplane distance
in meters, higher is better, threshold 8 m, δ = 0.5 m, α = 0.05, 3 throws per
arm (the counting rule reaches p = 0.05 with 3 against 3). All moves are
reasonable, and the frontier has only 4 neighbors, so no sample round is
needed. No factor is ordered, so steps are already at their finest.

Round 1. Incumbent light/dart/no.

| configuration | throws (m) |
|---|---|
| light/dart/no (incumbent) | 5.1, 5.4, 4.9 |
| heavy/dart/no | 4.2, 4.6, 4.4 |
| light/glider/no | 6.3, 6.0, 6.6 |
| light/delta/no | 5.0, 5.6, 5.2 |
| light/dart/yes | 5.8, 5.5, 6.1 |

Move test, every neighbor throw minus 0.5 above every incumbent throw (5.4
is the longest):

1. light/glider/no: 5.8, 5.5, 6.1, all above 5.4. Beats by δ, p = 0.05.
2. light/dart/yes: 5.3, 5.0, 5.6. 5.0 is below 5.1. Does not.
3. The other two do not.

Move to light/glider/no.

Round 2. Incumbent light/glider/no, re-measured.

| configuration | throws (m) |
|---|---|
| light/glider/no (incumbent) | 6.2, 6.5, 6.1 |
| heavy/glider/no | 5.0, 5.3, 4.8 |
| light/dart/no | 5.2, 4.9, 5.3 |
| light/delta/no | 5.5, 5.9, 5.6 |
| light/glider/yes | 6.4, 6.0, 6.3 |

No neighbor minus 0.5 clears 6.5, so no move. Verdict tests:

4. Worse (every neighbor throw below 6.1): heavy/glider (longest 5.3),
   light/dart (5.3), light/delta (5.9) are worse, p = 0.05 each.
   light/glider/yes (6.4) is not shown worse.
5. Not better by δ (every neighbor throw below every incumbent throw plus
   0.5, the lowest being 6.6): all four pass, p = 0.05 each.

Start 1 verdict: PLATEAU at 6.1 to 6.5 m, below the 8 m threshold. The nose
clip neither helps nor hurts by half a meter, so the cheaper plane (no clip)
is kept. Next: start 2 at the configuration farthest from the baseline,
heavy/delta/yes (distance 3).

Said to a student: "From my best plane I tried every single change. Three
were shorter on every throw. The paper clip was about the same."

## Plant strains

Dry biomass per plant (g), higher is better. Threshold 15% over the current
strain, δ = 0.5 g. Factors: strain (5, attributed by breeding family), light
hours (12 to 18, step 1, ordered), nitrogen dose (4 levels, ordered). Blocks:
greenhouse benches. Guardrail: death rate above 10% in a configuration.
Reasonable moves: one step of light or nitrogen; strain swaps go to the
frontier sample. Rounds are growth cycles, so starts run in parallel on
separate benches. Use the t-test rule once replicates exceed 5 per arm.

## Advertiser

Conversion rate, higher is better. Threshold +0.3 points over the current
ad, δ = 0.3 points. Factors: headline (20, attributed by length, reading
grade level, idiom; new headlines can be written to order), image (3), call
to action (3). Guardrails: unsubscribe and complaint rates. Pass-rate rule,
fixed n per round, read once. Starts are parallel traffic slices.
Confirmation is a holdout week.
