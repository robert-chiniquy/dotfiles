# File layouts

One directory per experiment. All files are markdown. Values a program reads
are in fenced YAML blocks and markdown tables: `PROTOCOL.md` and each
`DECISIONS.md` section have one YAML block, `NOISE.md` has one table, and a
round file has one YAML header and one table. Read them with a YAML library
and a markdown parser with table support.

## PROTOCOL.md

Prose: the question, the measurement procedure, blinding, guardrails. Then
one YAML block:

```yaml
version: 1
objective:
  kind: number          # number | pass_rate | preference
  direction: higher     # higher | lower (number only)
  unit: m
threshold:
  absolute: 8.0         # or margin_over_baseline: 0.3
delta: 0.5              # win-rate margin when kind is preference
alpha: 0.05
coverage_q: 0.2
budget:
  runs: 60
  batch: 15
blocks: [saturday, sunday]
factors:
  - name: paper
    kind: unordered     # ordered | unordered | integer | continuous
    values: [light, heavy]
  - name: fold
    kind: unordered
    values: [dart, glider, delta]
  - name: nose_clip
    kind: unordered
    values: ["no", "yes"]
infeasible: []
baseline: {paper: light, fold: dart, nose_clip: "no"}
reasonable_moves: all   # or a list of factor names whose one-step moves count
stop_m: 3               # optional; new starts stop after this many land in known basins
guardrails:
  - throw indoors only
```

An attributed factor lists levels with attribute values:

```yaml
factors:
  - name: headline
    kind: unordered
    attributes:
      - {name: length_words, kind: ordered, values: [short, medium, long], standard: "word count bands set in this protocol"}
      - {name: idiom, kind: unordered, values: [question, imperative, statement], standard: "sentence mood"}
    levels:
      - {id: h01, text: "Ready to save?", length_words: short, idiom: question}
      - {id: h02, text: "Save today", length_words: short, idiom: imperative}
    can_produce_levels: true
```

Distance between configurations: per factor, steps apart divided by steps in
the span (ordered), 0 or 1 (unordered), or the fraction of attributes that
differ (attributed); summed over factors.

## ROUND_<n>.md

A title, a sentence or two for the person running it, one YAML header, then
one table with rows in run order. The person running it fills in `result`,
`time`, `notes`.

```yaml
round: 2
start: 1
incumbent: light/glider/no
purpose: reasonable_neighbors   # reasonable_neighbors | frontier_sample | confirmation
steps: {}                       # current step per ordered factor, e.g. {light_hours: 2}
seed: 8812                      # run order and frontier sample
guardrails:
  - throw indoors only
```

| run | block | paper | fold | nose_clip | result | time | notes |
|---|---|---|---|---|---|---|---|
| r1-01 | saturday | light | glider | no | | | |
| r1-02 | saturday | light | dart | no | | | |

For a preference objective, each row is one duel: columns `left`, `right`
(configuration ids), and `winner`.

## NOISE.md

Phase 1 baseline runs, in the round table layout with no YAML header. These
runs are not counted against the search budget's rounds.

| run | block | paper | fold | nose_clip | result | time | notes |
|---|---|---|---|---|---|---|---|
| n1 | saturday | light | dart | no | 5.0 | 09:00 | |
| n2 | sunday | light | dart | no | 4.8 | 09:00 | |

## DECISIONS.md

Append one section per round; never edit earlier sections. Each holds a YAML
block and a few sentences reading it.

```yaml
round: 1
start: 1
rule: counting          # counting | welch | paired_t | binomial | one_sample_t
comparisons:
  # test: beats_by_delta | not_better_by_delta | worse | meets_threshold
  - {neighbor: "light/glider/no", test: beats_by_delta, p: 0.05, reject: true}
  - {neighbor: "light/dart/yes", test: beats_by_delta, p: null, reject: false}
dropped_runs: []
move: "light/glider/no"
steps: unchanged        # or a map from factor to new step, e.g. {light_hours: 2}
verdict: null
```

A round that reaches a verdict writes it as a mapping:

```yaml
verdict:
  kind: PLATEAU         # SOLVED | LOCAL_OPTIMUM | PLATEAU | BUDGET | DUPLICATE
  incumbent: light/glider/no
  delta: 0.5
  alpha: 0.05
  coverage_q: 0.2
  frontier_n: 14
  not_shown_worse: [light/glider/yes]
  unresolved: []
```
