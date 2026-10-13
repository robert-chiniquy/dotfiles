# Comparison rules

Every test is one-sided at α and rejects when p ≤ α. Record which rule each
decision used.

## Numbers, small n (counting rule)

Complete separation between two groups is an exact one-sided permutation
test. With a runs in one group and b in the other, the chance that every run
of one group beats every run of the other when there is no difference is
1 / C(a + b, a).

| runs per group | p |
|---|---|
| 3 and 3 | 1/20 = 0.05 |
| 3 and 4 | 1/35 ≈ 0.029 |
| 4 and 4 | 1/70 ≈ 0.014 |
| 5 and 5 | 1/252 ≈ 0.004 |

With 3 runs each, α below 0.05 cannot be reached; use 4 or more. By hand,
record p only when the groups separate; otherwise record that they did not
and the test does not reject.

For neighbor N and incumbent I:

1. N beats I by δ (move): every N run minus δ is above every I run.
2. N is not better by δ: every N run is below every I run plus δ.
3. N is worse: every N run is below every I run.

Said to a student: "Every new throw went farther than every old throw, even
after taking half a meter off each new one."

## Numbers, larger n

Shift by δ as above, then Welch's t-test; a paired t-test on block
differences when runs are paired by block; a permutation test on skewed
data.

## Pass rates

Exact test on two proportions; exact McNemar when runs are paired by task or
block.

## Pairwise preference

δ is a win-rate margin. N beats I when a one-sided binomial test rejects
"N's win rate ≤ 0.5 + δ". Not better by δ: rejects "win rate ≥ 0.5 + δ".
Worse: rejects "win rate ≥ 0.5". Randomize which side each is shown on and
record it. Do not assume transitivity.

## Repeated looks

Online tests read daily need an always-valid test (confidence sequence).
Until the tool supports one, fix n per round and read once. Check the
traffic split matches the plan before reading results.
